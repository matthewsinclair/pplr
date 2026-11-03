// pplr-contacts: the Contacts sidecar behind `pplr sync` (ST0001 design D2).
//
// It reads and saves Contacts and decides nothing: matching, planning and
// every write to a card are pplr's, which writes cards through Contacts.app
// scripts and sips. pplr runs it from beside its own binary.
//
// Usage:
//   pplr-contacts dump                     every card as JSON
//   pplr-contacts backup --backup-dir DIR  every card to a dated .vcf in DIR
//   pplr-contacts group [--group NAME]     the group's id in the default
//                                          account (iCloud), made if missing
//
// A card linked across accounts comes back from the unified fetch with a
// merged id that Contacts.app cannot address, so `dump` resolves each card to
// a real one, iCloud first. Without access to Contacts it exits 3, saying
// where to allow it. A backup leaves out the notes: macOS does not give them
// to an unentitled tool.

import Contacts
import Foundation

// MARK: - Model

struct Card: Codable {
    var id: String
    var given: String
    var family: String
    var organization: String
    var jobTitle: String
    var emails: [String]
    var phones: [String]
    var urls: [LabelledValue]
    var linkedin: [String]
    var groups: [String]
    var account: String?       // "home" (iCloud, where the group lives), "other", or "none"; absent means home
    var hasImage: Bool?        // a photo or thumbnail; absent means unknown (treated as none)
}

struct LabelledValue: Codable {
    var label: String
    var value: String
}

// MARK: - Normalisation

func fold(_ s: String) -> String {
    s.folding(options: [.caseInsensitive, .diacriticInsensitive], locale: nil)
        .trimmingCharacters(in: .whitespacesAndNewlines)
}

/// Digits with a leading +, UK numbers without a country code assumed +44,
/// and the trunk 0 dropped after a country code, written "(0)" or not.
func normalisePhone(_ raw: String) -> String {
    var s = raw.replacingOccurrences(of: "(0)", with: "")
    s = String(s.unicodeScalars.filter { CharacterSet(charactersIn: "+0123456789").contains($0) }.map(Character.init))
    if s.hasPrefix("00") { s = "+" + s.dropFirst(2) }
    if s.hasPrefix("0") { s = "+44" + s.dropFirst(1) }
    if !s.hasPrefix("+") && !s.isEmpty { s = "+" + s }
    // Italy keeps its 0 after +39; these codes never do
    for cc in ["+44", "+61", "+33", "+49", "+32", "+31", "+353", "+64", "+262"] where s.hasPrefix(cc + "0") {
        s = cc + s.dropFirst(cc.count + 1)
    }
    return s
}

/// "https://www.linkedin.com/in/JonTolley/?x" -> "jontolley"
func linkedinSlug(_ raw: String) -> String? {
    let lower = raw.lowercased()
    guard let r = lower.range(of: "linkedin.com/in/") else { return nil }
    let rest = lower[r.upperBound...]
    let slug = rest.split(whereSeparator: { "/?#)] ".contains($0) }).first.map(String.init) ?? ""
    return slug.isEmpty ? nil : (slug.removingPercentEncoding ?? slug)
}

// MARK: - Contacts side

func requestAccess(_ store: CNContactStore) -> Bool {
    switch CNContactStore.authorizationStatus(for: .contacts) {
    case .authorized: return true
    case .notDetermined:
        let sem = DispatchSemaphore(value: 0)
        var ok = false
        store.requestAccess(for: .contacts) { granted, _ in ok = granted; sem.signal() }
        sem.wait()
        return ok
    default: return false
    }
}

func openStore() -> CNContactStore {
    let store = CNContactStore()
    guard requestAccess(store) else {
        FileHandle.standardError.write(Data("""
        pplr sync: no access to Contacts.
        Allow it in System Settings > Privacy & Security > Contacts for the app running pplr (eg iTerm or Terminal), then run it again.\n
        """.utf8))
        exit(3)
    }
    return store
}

func loadCards() throws -> [Card] {
    let store = openStore()
    var membership: [String: [String]] = [:]
    for g in try store.groups(matching: nil) {
        let ids = try store.unifiedContacts(matching: CNContact.predicateForContactsInGroup(withIdentifier: g.identifier),
                                            keysToFetch: [CNContactIdentifierKey as CNKeyDescriptor])
        for c in ids { membership[c.identifier, default: []].append(g.name) }
    }
    let keys: [CNKeyDescriptor] = [CNContactIdentifierKey, CNContactGivenNameKey, CNContactFamilyNameKey,
                                   CNContactOrganizationNameKey, CNContactJobTitleKey, CNContactEmailAddressesKey,
                                   CNContactPhoneNumbersKey, CNContactUrlAddressesKey, CNContactSocialProfilesKey,
                                   CNContactImageDataAvailableKey, CNContactThumbnailImageDataKey]
        .map { $0 as CNKeyDescriptor }
    // The PPLR group lives in the default account (iCloud) only; cards in
    // other accounts are marked by the pplr URL alone
    let home = store.defaultContainerIdentifier()
    // The real cards, account by account. A person whose cards are linked
    // across accounts comes back from the unified fetch with a merged id
    // (no ":ABPerson"), which Contacts.app cannot address; such a card is
    // resolved below to a real one, iCloud first.
    struct Real { var id: String; var account: String; var emails: Set<String>; var name: String }
    var reals: [Real] = []
    for container in try store.containers(matching: nil) {
        let req = CNContactFetchRequest(keysToFetch: [CNContactIdentifierKey, CNContactEmailAddressesKey,
                                                      CNContactGivenNameKey, CNContactFamilyNameKey].map { $0 as CNKeyDescriptor })
        req.unifyResults = false
        req.predicate = CNContact.predicateForContactsInContainer(withIdentifier: container.identifier)
        try store.enumerateContacts(with: req) { c, _ in
            reals.append(Real(id: c.identifier, account: container.identifier == home ? "home" : "other",
                              emails: Set(c.emailAddresses.map { ($0.value as String).lowercased() }),
                              name: fold("\(c.givenName) \(c.familyName)")))
        }
    }
    let account = Dictionary(reals.map { ($0.id, $0.account) }, uniquingKeysWith: { a, _ in a })
    func resolve(_ id: String, _ emails: [String], _ name: String) -> (String, String) {
        if let a = account[id] { return (id, a) }
        let wanted = Set(emails)
        // Same name first (a shared or office email can belong to someone
        // else's card), then a shared email
        let sameName = reals.filter { $0.name == fold(name) }
        let both = sameName.filter { !$0.emails.isDisjoint(with: wanted) }
        let candidates = !both.isEmpty ? both : !sameName.isEmpty ? sameName : reals.filter { !$0.emails.isDisjoint(with: wanted) }
        guard let pick = candidates.first(where: { $0.account == "home" }) ?? candidates.first else { return (id, "none") }
        return (pick.id, pick.account)
    }
    var cards: [Card] = []
    try store.enumerateContacts(with: CNContactFetchRequest(keysToFetch: keys)) { c, _ in
        let urls = c.urlAddresses.map { LabelledValue(label: $0.label.map { CNLabeledValue<NSString>.localizedString(forLabel: $0) } ?? "", value: $0.value as String) }
        var li = urls.compactMap { linkedinSlug($0.value) }
        for sp in c.socialProfiles {
            let v = sp.value
            if let s = linkedinSlug(v.urlString) { li.append(s) }
            else if v.service.lowercased() == "linkedin", !v.username.isEmpty { li.append(v.username.lowercased()) }
        }
        let emails = c.emailAddresses.map { ($0.value as String).lowercased() }
        let (realId, acct) = resolve(c.identifier, emails, "\(c.givenName) \(c.familyName)")
        cards.append(Card(id: realId, given: c.givenName, family: c.familyName,
                          organization: c.organizationName, jobTitle: c.jobTitle,
                          emails: c.emailAddresses.map { ($0.value as String).lowercased() },
                          phones: c.phoneNumbers.map { normalisePhone($0.value.stringValue) },
                          urls: urls, linkedin: Array(Set(li)).sorted(), groups: membership[c.identifier] ?? [],
                          account: acct, hasImage: c.imageDataAvailable || (c.thumbnailImageData?.isEmpty == false)))
    }
    return cards
}

// MARK: - Backup and the group

func stamp() -> String {
    let f = DateFormatter(); f.dateFormat = "yyyyMMdd-HHmmss"; return f.string(from: Date())
}

/// Every card, before anything is written. Returns the file written.
func backup(to dir: String) throws -> String {
    try FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
    let store = openStore()
    let keys: [CNKeyDescriptor] = [CNContactVCardSerialization.descriptorForRequiredKeys(),
                                   CNContactImageDataKey as CNKeyDescriptor]
    var all: [CNContact] = []
    try store.enumerateContacts(with: CNContactFetchRequest(keysToFetch: keys)) { c, _ in all.append(c) }
    let path = (dir as NSString).appendingPathComponent("\(stamp())-contacts.vcf")
    try CNContactVCardSerialization.data(with: all).write(to: URL(fileURLWithPath: path))
    return path
}

/// The pplr group in the default account (iCloud), made if it is missing
func homeGroupID(_ store: CNContactStore, _ group: String) throws -> String {
    let home = store.defaultContainerIdentifier()
    let inHome = { try store.groups(matching: CNGroup.predicateForGroupsInContainer(withIdentifier: home)) }
    if let g = try inHome().first(where: { $0.name == group }) { return g.identifier }
    let g = CNMutableGroup(); g.name = group
    let req = CNSaveRequest(); req.add(g, toContainerWithIdentifier: home)
    try store.execute(req)
    guard let made = try inHome().first(where: { $0.name == group }) else {
        throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "made the \(group) group in Contacts, but cannot find it"])
    }
    return made.identifier
}

// MARK: - Main

var args = Array(CommandLine.arguments.dropFirst())
func take(_ flag: String) -> String? {
    guard let i = args.firstIndex(of: flag), i + 1 < args.count else { return nil }
    let v = args[i + 1]; args.removeSubrange(i...i + 1); return v
}

let cmd = args.isEmpty ? "" : args.removeFirst()
let group = take("--group") ?? "PPLR"
let backupDir = take("--backup-dir")

do {
    switch cmd {
    case "dump":
        let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        FileHandle.standardOutput.write(try enc.encode(try loadCards()) + Data("\n".utf8))
    case "backup":
        guard let backupDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "backup needs --backup-dir"]) }
        print("Backup: \(try backup(to: backupDir))")
    case "group":
        print(try homeGroupID(openStore(), group))
    default:
        FileHandle.standardError.write(Data("usage: pplr-contacts dump\n       pplr-contacts backup --backup-dir DIR\n       pplr-contacts group [--group NAME]\n".utf8))
        exit(2)
    }
} catch {
    FileHandle.standardError.write(Data("pplr-contacts: \(error.localizedDescription)\n".utf8))
    exit(1)
}
