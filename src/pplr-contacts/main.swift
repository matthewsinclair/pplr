// pplr-contacts: the Apple Contacts engine behind `pplr sync`.
//
// `check` compares the pplr people tree with Contacts and reports who is
// linked, who matches, who is missing on each side, and which schema fields
// differ. `link` marks matched cards as pplr's, and changes nothing else.
// Only the header fields of each About file take part: name, Role, Company,
// Email, Phone and LinkedIn. Notes, bios and meetings never leave pplr.
//
// Provenance in Contacts: membership of the "PPLR" group, plus a URL labelled
// "pplr" whose value is pplr://<Letter>/<Surname,%20First>, which is also the
// stable link back to the person's folder. `link` adds both, and writes
// About/<First Surname> (Contacts).webloc opening addressbook://<card id>.
//
// Usage:
//   pplr-contacts check --people-dir DIR [--group NAME] [--json] [--verbose]
//                       [--contacts-json FILE]
//   pplr-contacts link  --people-dir DIR [--group NAME] [--apply]
//                       [--name "Surname, First"]... [--all-names]
//                       [--backup-dir DIR] [--contacts-json FILE]
//   pplr-contacts backup --backup-dir DIR [--contacts-json FILE]
//   pplr-contacts dump [--group NAME]
//
// `link` is a dry run unless --apply is given. It links the people matched by
// email or LinkedIn, plus the name-only matches named with --name (or all of
// them with --all-names); ambiguous people are never linked. Before writing it
// saves every card to a dated .vcf in --backup-dir (notes excluded: macOS does
// not give them to an unentitled tool).
//
// --contacts-json reads contacts from a file in the `dump` format instead of
// the Contacts store, so the tests never need Contacts access.

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
}

struct LabelledValue: Codable {
    var label: String
    var value: String
}

struct Person {
    var key: String         // "K/Kemp, Jon"
    var given: String
    var family: String
    var role: String
    var company: String
    var emails: [String]
    var phones: [String]
    var linkedin: String
    var aboutPath: String
    var personDir: String
}

// MARK: - Normalisation

func fold(_ s: String) -> String {
    s.folding(options: [.caseInsensitive, .diacriticInsensitive], locale: nil)
        .trimmingCharacters(in: .whitespacesAndNewlines)
}

/// Digits with a leading +, UK numbers without a country code assumed +44,
/// and the "(0)" trunk prefix dropped.
func normalisePhone(_ raw: String) -> String {
    var s = raw.replacingOccurrences(of: "(0)", with: "")
    s = String(s.unicodeScalars.filter { CharacterSet(charactersIn: "+0123456789").contains($0) }.map(Character.init))
    if s.hasPrefix("00") { s = "+" + s.dropFirst(2) }
    if s.hasPrefix("0") { s = "+44" + s.dropFirst(1) }
    if !s.hasPrefix("+") && !s.isEmpty { s = "+" + s }
    return s
}

/// "https://www.linkedin.com/in/JonTolley/?x" -> "jontolley"
func linkedinSlug(_ raw: String) -> String? {
    let lower = raw.lowercased()
    guard let r = lower.range(of: "linkedin.com/in/") else { return nil }
    let rest = lower[r.upperBound...]
    let slug = rest.split(whereSeparator: { "/?#".contains($0) }).first.map(String.init) ?? ""
    return slug.isEmpty ? nil : (slug.removingPercentEncoding ?? slug)
}

// MARK: - pplr side

/// Markdown link text, or the plain value: "[Co-Founder](https://x)" -> "Co-Founder"
func linkText(_ v: String) -> String {
    let t = v.trimmingCharacters(in: .whitespaces)
    if t.hasPrefix("["), let close = t.range(of: "](") {
        return String(t[t.index(after: t.startIndex)..<close.lowerBound]).trimmingCharacters(in: .whitespaces)
    }
    return t
}

func matches(_ pattern: String, in s: String) -> [String] {
    guard let re = try? NSRegularExpression(pattern: pattern) else { return [] }
    let ns = s as NSString
    return re.matches(in: s, range: NSRange(location: 0, length: ns.length)).map { ns.substring(with: $0.range) }
}

func loadPeople(_ root: String) -> [Person] {
    let fm = FileManager.default
    var people: [Person] = []
    let letters = ((try? fm.contentsOfDirectory(atPath: root)) ?? [])
        .filter { $0.count == 1 && $0.first!.isUppercase }.sorted()
    for letter in letters {
        let ldir = (root as NSString).appendingPathComponent(letter)
        let dirs = ((try? fm.contentsOfDirectory(atPath: ldir)) ?? []).filter { $0.contains(", ") }.sorted()
        for name in dirs {
            let pdir = (ldir as NSString).appendingPathComponent(name)
            let parts = name.components(separatedBy: ", ")
            var p = Person(key: "\(letter)/\(name)", given: parts.dropFirst().joined(separator: ", "),
                           family: parts[0], role: "", company: "", emails: [], phones: [], linkedin: "", aboutPath: "", personDir: pdir)
            let adir = (pdir as NSString).appendingPathComponent("About")
            if let about = ((try? fm.contentsOfDirectory(atPath: adir)) ?? []).sorted().first(where: { $0.hasSuffix("(About).md") }) {
                p.aboutPath = (adir as NSString).appendingPathComponent(about)
                let text = (try? String(contentsOfFile: p.aboutPath, encoding: .utf8)) ?? ""
                var seen = Set<String>()
                for line in text.components(separatedBy: .newlines) {
                    guard line.hasPrefix("- "), let colon = line.firstIndex(of: ":") else { continue }
                    let field = line[line.index(line.startIndex, offsetBy: 2)..<colon].trimmingCharacters(in: .whitespaces)
                    guard !seen.contains(field) else { continue }
                    let value = String(line[line.index(after: colon)...])
                    switch field {
                    case "Role": p.role = linkText(value)
                    case "Company": p.company = linkText(value)
                    case "Email":
                        p.emails = Array(Set(matches("[A-Za-z0-9._%+'-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}", in: value).map { $0.lowercased() })).sorted()
                    case "Phone":
                        p.phones = value.components(separatedBy: CharacterSet(charactersIn: ",;"))
                            .map(normalisePhone).filter { $0.count > 6 }
                    case "LinkedIn": p.linkedin = linkedinSlug(value) ?? ""
                    default: continue
                    }
                    seen.insert(field)
                }
            }
            people.append(p)
        }
    }
    return people
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

func loadCards(fixture: String?) throws -> [Card] {
    if let fixture {
        return try JSONDecoder().decode([Card].self, from: Data(contentsOf: URL(fileURLWithPath: fixture)))
    }
    let store = openStore()
    var membership: [String: [String]] = [:]
    for g in try store.groups(matching: nil) {
        let ids = try store.unifiedContacts(matching: CNContact.predicateForContactsInGroup(withIdentifier: g.identifier),
                                            keysToFetch: [CNContactIdentifierKey as CNKeyDescriptor])
        for c in ids { membership[c.identifier, default: []].append(g.name) }
    }
    let keys: [CNKeyDescriptor] = [CNContactIdentifierKey, CNContactGivenNameKey, CNContactFamilyNameKey,
                                   CNContactOrganizationNameKey, CNContactJobTitleKey, CNContactEmailAddressesKey,
                                   CNContactPhoneNumbersKey, CNContactUrlAddressesKey, CNContactSocialProfilesKey]
        .map { $0 as CNKeyDescriptor }
    var cards: [Card] = []
    try store.enumerateContacts(with: CNContactFetchRequest(keysToFetch: keys)) { c, _ in
        let urls = c.urlAddresses.map { LabelledValue(label: $0.label.map { CNLabeledValue<NSString>.localizedString(forLabel: $0) } ?? "", value: $0.value as String) }
        var li = urls.compactMap { linkedinSlug($0.value) }
        for sp in c.socialProfiles {
            let v = sp.value
            if let s = linkedinSlug(v.urlString) { li.append(s) }
            else if v.service.lowercased() == "linkedin", !v.username.isEmpty { li.append(v.username.lowercased()) }
        }
        cards.append(Card(id: c.identifier, given: c.givenName, family: c.familyName,
                          organization: c.organizationName, jobTitle: c.jobTitle,
                          emails: c.emailAddresses.map { ($0.value as String).lowercased() },
                          phones: c.phoneNumbers.map { normalisePhone($0.value.stringValue) },
                          urls: urls, linkedin: Array(Set(li)).sorted(), groups: membership[c.identifier] ?? []))
    }
    return cards
}

// MARK: - Check

struct FieldDiff: Codable {
    var field: String
    var pplr: [String]
    var contacts: [String]
}

struct Pair: Codable {
    var person: String
    var contact: String
    var contactName: String
    var how: String            // linked | email | linkedin | name
    var inGroup: Bool
    var hasMarker: Bool
    var webloc: String         // About/<First Surname> (Contacts).webloc
    var hasWebloc: Bool        // exists and opens this card
    var diffs: [FieldDiff]
}

struct Report: Codable {
    var group: String
    var people: Int
    var contacts: Int
    var groupMembers: Int
    var linked: [Pair]
    var matched: [Pair]
    var nameOnly: [Pair]
    var ambiguous: [String: [String]]
    var pplrOnly: [String]
    var groupOrphans: [String]
}

/// Opens Contacts.app on the card (macOS; the card id is this Mac's)
func contactsURL(_ id: String) -> String { "addressbook://\(id)" }

func weblocPath(_ p: Person) -> String {
    let about = p.aboutPath.isEmpty
        ? ((p.personDir as NSString).appendingPathComponent("About"))
        : (p.aboutPath as NSString).deletingLastPathComponent
    return (about as NSString).appendingPathComponent("\(p.given) \(p.family) (Contacts).webloc")
}

func weblocOpens(_ path: String, _ id: String) -> Bool {
    guard let d = FileManager.default.contents(atPath: path),
          let plist = try? PropertyListSerialization.propertyList(from: d, format: nil) as? [String: Any] else { return false }
    return plist["URL"] as? String == contactsURL(id)
}

func writeWebloc(_ path: String, _ id: String) throws {
    try FileManager.default.createDirectory(atPath: (path as NSString).deletingLastPathComponent, withIntermediateDirectories: true)
    let d = try PropertyListSerialization.data(fromPropertyList: ["URL": contactsURL(id)], format: .xml, options: 0)
    try d.write(to: URL(fileURLWithPath: path))
}

func pplrMarker(_ c: Card) -> String? {
    c.urls.first(where: { $0.label.lowercased() == "pplr" || $0.value.hasPrefix("pplr://") })
        .map { ($0.value.replacingOccurrences(of: "pplr://", with: "").removingPercentEncoding ?? $0.value) }
}

func displayName(_ c: Card) -> String {
    let n = [c.given, c.family].filter { !$0.isEmpty }.joined(separator: " ")
    return n.isEmpty ? (c.organization.isEmpty ? "(no name)" : c.organization) : n
}

func diffs(_ p: Person, _ c: Card) -> [FieldDiff] {
    var out: [FieldDiff] = []
    func scalar(_ f: String, _ a: String, _ b: String) {
        if fold(a) != fold(b) { out.append(FieldDiff(field: f, pplr: a.isEmpty ? [] : [a], contacts: b.isEmpty ? [] : [b])) }
    }
    func set(_ f: String, _ a: [String], _ b: [String]) {
        let sa = Set(a), sb = Set(b)
        if sa != sb { out.append(FieldDiff(field: f, pplr: sa.subtracting(sb).sorted(), contacts: sb.subtracting(sa).sorted())) }
    }
    scalar("name", "\(p.given) \(p.family)", "\(c.given) \(c.family)")
    scalar("company", p.company, c.organization)
    scalar("role", p.role, c.jobTitle)
    set("email", p.emails, c.emails)
    set("phone", p.phones, c.phones)
    set("linkedin", p.linkedin.isEmpty ? [] : [p.linkedin], c.linkedin)
    return out
}

func check(people: [Person], cards: [Card], group: String) -> Report {
    let byId = Dictionary(uniqueKeysWithValues: cards.map { ($0.id, $0) })
    var byMarker: [String: Card] = [:], byEmail: [String: [Card]] = [:], bySlug: [String: [Card]] = [:], byName: [String: [Card]] = [:]
    for c in cards {
        if let m = pplrMarker(c) { byMarker[m] = c }
        for e in c.emails { byEmail[e, default: []].append(c) }
        for s in c.linkedin { bySlug[s, default: []].append(c) }
        byName[fold("\(c.given) \(c.family)"), default: []].append(c)
    }
    var r = Report(group: group, people: people.count, contacts: cards.count,
                   groupMembers: cards.filter { $0.groups.contains(group) }.count,
                   linked: [], matched: [], nameOnly: [], ambiguous: [:], pplrOnly: [], groupOrphans: [])
    var claimed = Set<String>()
    func uniq(_ cs: [Card]) -> [Card] { var seen = Set<String>(); return cs.filter { seen.insert($0.id).inserted } }
    func pair(_ p: Person, _ c: Card, _ how: String) -> Pair {
        claimed.insert(c.id)
        let w = weblocPath(p)
        return Pair(person: p.key, contact: c.id, contactName: displayName(c), how: how, inGroup: c.groups.contains(group),
                    hasMarker: pplrMarker(c) == p.key, webloc: w, hasWebloc: weblocOpens(w, c.id), diffs: diffs(p, c))
    }
    for p in people {
        if let c = byMarker[p.key] { r.linked.append(pair(p, c, "linked")); continue }
        let viaEmail = uniq(p.emails.flatMap { byEmail[$0] ?? [] })
        let viaSlug = p.linkedin.isEmpty ? [] : uniq(bySlug[p.linkedin] ?? [])
        let viaName = byName[fold("\(p.given) \(p.family)")] ?? []
        if viaEmail.count == 1 { r.matched.append(pair(p, viaEmail[0], "email")) }
        else if viaEmail.count > 1 { r.ambiguous[p.key] = viaEmail.map { "\(displayName($0)) (\($0.id))" } }
        else if viaSlug.count == 1 { r.matched.append(pair(p, viaSlug[0], "linkedin")) }
        else if viaName.count == 1 { r.nameOnly.append(pair(p, viaName[0], "name")) }
        else if viaName.count > 1 { r.ambiguous[p.key] = viaName.map { "\(displayName($0)) (\($0.id))" } }
        else { r.pplrOnly.append(p.key) }
    }
    r.groupOrphans = cards.filter { $0.groups.contains(group) && !claimed.contains($0.id) }.map { displayName($0) }.sorted()
    _ = byId
    return r
}

// MARK: - Link

/// pplr://K/Kemp,%20Jon
func markerURL(_ key: String) -> String {
    "pplr://" + (key.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed) ?? key)
}

/// "Kemp, Jon" or "K/Kemp, Jon" -> "K/Kemp, Jon"
func personKey(_ name: String) -> String {
    name.contains("/") ? name : "\(name.prefix(1).uppercased())/\(name)"
}

struct LinkPlan: Codable {
    var toLink: [Pair]          // the pairs that need a marker, the group, or both
    var alreadyDone: Int
    var namesSkipped: [String]  // name-only matches not confirmed
    var unknownNames: [String]  // --name values that are not a name-only match
    var ambiguous: [String]
}

func planLink(_ r: Report, names: [String], allNames: Bool) -> LinkPlan {
    let wanted = Set(names.map(personKey))
    let nameKeys = Set(r.nameOnly.map(\.person))
    let confirmed = r.nameOnly.filter { allNames || wanted.contains($0.person) }
    let all = r.linked + r.matched + confirmed
    let done = { (p: Pair) in p.hasMarker && p.inGroup && p.hasWebloc }
    return LinkPlan(toLink: all.filter { !done($0) },
                    alreadyDone: all.filter(done).count,
                    namesSkipped: r.nameOnly.filter { !(allNames || wanted.contains($0.person)) }.map(\.person),
                    unknownNames: wanted.subtracting(nameKeys).subtracting((r.linked + r.matched).map(\.person)).sorted(),
                    ambiguous: r.ambiguous.keys.sorted())
}

func stamp() -> String {
    let f = DateFormatter(); f.dateFormat = "yyyyMMdd-HHmmss"; return f.string(from: Date())
}

/// Every card, before anything is written. Returns the file written.
func backup(to dir: String, fixture: String?) throws -> String {
    try FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
    if let fixture {
        let path = (dir as NSString).appendingPathComponent("\(stamp())-contacts.json")
        try FileManager.default.copyItem(atPath: fixture, toPath: path)
        return path
    }
    let store = openStore()
    let keys: [CNKeyDescriptor] = [CNContactVCardSerialization.descriptorForRequiredKeys(),
                                   CNContactImageDataKey as CNKeyDescriptor]
    var all: [CNContact] = []
    try store.enumerateContacts(with: CNContactFetchRequest(keysToFetch: keys)) { c, _ in all.append(c) }
    let path = (dir as NSString).appendingPathComponent("\(stamp())-contacts.vcf")
    try CNContactVCardSerialization.data(with: all).write(to: URL(fileURLWithPath: path))
    return path
}

func applyFixture(_ plan: LinkPlan, group: String, fixture: String, limit: Int?) throws {
    var cards = try JSONDecoder().decode([Card].self, from: Data(contentsOf: URL(fileURLWithPath: fixture)))
    for p in plan.toLink.prefix(limit ?? Int.max) {
        guard let i = cards.firstIndex(where: { $0.id == p.contact }) else { continue }
        if !p.hasMarker { cards[i].urls.append(LabelledValue(label: "pplr", value: markerURL(p.person))) }
        if !cards[i].groups.contains(group) { cards[i].groups.append(group) }
        if !p.hasWebloc { try writeWebloc(p.webloc, p.contact) }
    }
    let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
    try enc.encode(cards).write(to: URL(fileURLWithPath: fixture))
}

func applyLive(_ plan: LinkPlan, group: String, limit: Int?) throws {
    let store = openStore()
    var groups: [String: CNGroup] = [:]   // container id -> the group in it
    func groupIn(_ container: String) throws -> CNGroup {
        if let g = groups[container] { return g }
        let existing = try store.groups(matching: CNGroup.predicateForGroupsInContainer(withIdentifier: container))
        if let g = existing.first(where: { $0.name == group }) { groups[container] = g; return g }
        let g = CNMutableGroup(); g.name = group
        let req = CNSaveRequest(); req.add(g, toContainerWithIdentifier: container)
        try store.execute(req)
        let made = try store.groups(matching: CNGroup.predicateForGroupsInContainer(withIdentifier: container))
            .first(where: { $0.name == group })!
        groups[container] = made
        return made
    }
    var linked = 0, failed = 0
    defer { print("  written \(linked), failed \(failed)") }
    for p in plan.toLink.prefix(limit ?? Int.max) {
        guard let container = try store.containers(matching: CNContainer.predicateForContainerOfContact(withIdentifier: p.contact)).first else {
            print("  skipped \(p.person): no container for its card"); continue
        }
        // One change per save request: an update and a group membership in the
        // same request failed in CoreData (Cocoa error 134092) on the first card.
        let keys = [CNContactIdentifierKey, CNContactUrlAddressesKey].map { $0 as CNKeyDescriptor }
        do {
            if !p.hasMarker {
                let m = try store.unifiedContact(withIdentifier: p.contact, keysToFetch: keys).mutableCopy() as! CNMutableContact
                m.urlAddresses.append(CNLabeledValue(label: "pplr", value: markerURL(p.person) as NSString))
                let req = CNSaveRequest(); req.update(m)
                try store.execute(req)
            }
            if !p.inGroup {
                let c = try store.unifiedContact(withIdentifier: p.contact, keysToFetch: keys)
                let req = CNSaveRequest(); req.addMember(c, to: try groupIn(container.identifier))
                try store.execute(req)
            }
            if !p.hasWebloc { try writeWebloc(p.webloc, p.contact) }
            linked += 1
        } catch {
            print("  FAILED \(p.person): \(error.localizedDescription)")
            failed += 1
            if failed >= 3 { print("  stopping after three failures"); break }
        }
    }
}

func printPlan(_ plan: LinkPlan, group: String, applied: Bool, backupPath: String?) {
    print(applied ? "pplr sync --link (applied)" : "pplr sync --link (dry run: nothing written; --apply to write)")
    print("")
    print("  to link                 \(plan.toLink.count)")
    print("  already linked          \(plan.alreadyDone)")
    print("  name only, not linked   \(plan.namesSkipped.count)")
    print("  ambiguous, not linked   \(plan.ambiguous.count)")
    if !plan.toLink.isEmpty {
        print("\n\(applied ? "Linked" : "Would link") (the pplr URL and the \"\(group)\" group on the card, nothing else; a (Contacts).webloc in About):")
        for p in plan.toLink {
            var what: [String] = []
            if !p.hasMarker { what.append("pplr URL") }
            if !p.inGroup { what.append("group") }
            if !p.hasWebloc { what.append(".webloc") }
            print("  \(p.person)  <->  \(p.contactName)  [\(p.how); \(what.joined(separator: " + "))]")
        }
    }
    if !plan.unknownNames.isEmpty {
        print("\nNot a name-only match, so ignored:")
        plan.unknownNames.forEach { print("  \($0)") }
    }
    if !plan.namesSkipped.isEmpty {
        print("\nName-only matches left alone (confirm with --name \"Surname, First\", or --all-names):")
        plan.namesSkipped.forEach { print("  \($0)") }
    }
    if let backupPath { print("\nBackup: \(backupPath)") }
}

// MARK: - Output

func printReport(_ r: Report, verbose: Bool) {
    let withDiffs = (r.linked + r.matched).filter { !$0.diffs.isEmpty }
    print("pplr sync --check (read-only)")
    print("")
    print("  pplr people             \(r.people)")
    print("  Contacts cards          \(r.contacts)")
    print("  in group \(r.group.padding(toLength: 15, withPad: " ", startingAt: 0))\(r.groupMembers)")
    print("")
    print("  linked (pplr marker)    \(r.linked.count)")
    print("  matched by email        \(r.matched.filter { $0.how == "email" }.count)")
    print("  matched by LinkedIn     \(r.matched.filter { $0.how == "linkedin" }.count)")
    print("  name only (confirm)     \(r.nameOnly.count)")
    print("  ambiguous               \(r.ambiguous.count)")
    print("  only in pplr            \(r.pplrOnly.count)")
    print("  group orphans           \(r.groupOrphans.count)")
    print("  matched, fields differ  \(withDiffs.count)")

    func section(_ title: String, _ lines: [String]) {
        guard !lines.isEmpty else { return }
        print("\n\(title)")
        lines.forEach { print("  \($0)") }
    }
    func diffLines(_ ps: [Pair]) -> [String] {
        ps.flatMap { p -> [String] in
            let head = "\(p.person)  <->  \(p.contactName)  [\(p.how)\(p.inGroup ? ", in group" : "")]"
            let body = p.diffs.map { d -> String in
                let a = d.pplr.isEmpty ? "-" : d.pplr.joined(separator: ", ")
                let b = d.contacts.isEmpty ? "-" : d.contacts.joined(separator: ", ")
                return "    \(d.field.padding(toLength: 9, withPad: " ", startingAt: 0)) pplr: \(a)  |  Contacts: \(b)"
            }
            return [head] + body
        }
    }
    section("Name only: same name, no shared email or LinkedIn. Confirm before linking:",
            r.nameOnly.map { "\($0.person)  <->  \($0.contactName)" })
    section("Ambiguous: more than one card could be this person:",
            r.ambiguous.keys.sorted().map { "\($0): \(r.ambiguous[$0]!.joined(separator: "; "))" })
    section("Matched, with differences:", diffLines(withDiffs))
    section("In the \"\(r.group)\" group but not matched to anyone in pplr:", r.groupOrphans)
    if verbose { section("Only in pplr:", r.pplrOnly) }
    else if !r.pplrOnly.isEmpty { print("\n(\(r.pplrOnly.count) only in pplr; --verbose lists them)") }
}

// MARK: - Main

var args = Array(CommandLine.arguments.dropFirst())
func take(_ flag: String) -> String? {
    guard let i = args.firstIndex(of: flag), i + 1 < args.count else { return nil }
    let v = args[i + 1]; args.removeSubrange(i...i + 1); return v
}
func flag(_ f: String) -> Bool { if let i = args.firstIndex(of: f) { args.remove(at: i); return true }; return false }

let cmd = args.isEmpty ? "" : args.removeFirst()
let group = take("--group") ?? "PPLR"
let fixture = take("--contacts-json")
let peopleDir = take("--people-dir")
let asJSON = flag("--json")
let verbose = flag("--verbose") || flag("-v")
let apply = flag("--apply")
let allNames = flag("--all-names")
let limit = take("--limit").flatMap(Int.init)
let backupDir = take("--backup-dir")
var names: [String] = []
while let n = take("--name") { names.append(n) }

do {
    switch cmd {
    case "dump":
        let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        var cards = try loadCards(fixture: fixture)
        if args.contains("--only-group") { cards = cards.filter { $0.groups.contains(group) } }
        print(String(data: try enc.encode(cards), encoding: .utf8)!)
    case "check":
        guard let peopleDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "check needs --people-dir"]) }
        let report = check(people: loadPeople(peopleDir), cards: try loadCards(fixture: fixture), group: group)
        if asJSON {
            let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
            print(String(data: try enc.encode(report), encoding: .utf8)!)
        } else { printReport(report, verbose: verbose) }
    case "backup":
        guard let backupDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "backup needs --backup-dir"]) }
        print("Backup: \(try backup(to: backupDir, fixture: fixture))")
    case "link":
        guard let peopleDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "link needs --people-dir"]) }
        let report = check(people: loadPeople(peopleDir), cards: try loadCards(fixture: fixture), group: group)
        let plan = planLink(report, names: names, allNames: allNames)
        var backupPath: String? = nil
        if apply && !plan.toLink.isEmpty {
            guard let backupDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "--apply needs --backup-dir"]) }
            backupPath = try backup(to: backupDir, fixture: fixture)
            if let fixture { try applyFixture(plan, group: group, fixture: fixture, limit: limit) } else { try applyLive(plan, group: group, limit: limit) }
        }
        printPlan(plan, group: group, applied: apply, backupPath: backupPath)
    default:
        FileHandle.standardError.write(Data("usage: pplr-contacts check --people-dir DIR [--group NAME] [--json] [--verbose] [--contacts-json FILE]\n       pplr-contacts link --people-dir DIR [--group NAME] [--apply] [--name NAME]... [--all-names] [--backup-dir DIR]\n       pplr-contacts dump [--group NAME] [--only-group]\n".utf8))
        exit(2)
    }
} catch {
    FileHandle.standardError.write(Data("pplr-contacts: \(error.localizedDescription)\n".utf8))
    exit(1)
}
