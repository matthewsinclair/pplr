// pplr-contacts: the Apple Contacts engine behind `pplr sync`.
//
// `check` compares the pplr people tree with Contacts and reports who is
// linked, who matches, who is missing on each side, and which schema fields
// differ. `link` marks matched cards as pplr's, and changes nothing else.
// Only the header fields of each About file take part: name, Role, Company,
// Email, Phone and LinkedIn. Notes, bios and meetings never leave pplr.
//
// Provenance in Contacts: membership of the "PPLR" group, plus a URL labelled
// "pplr" whose value is pplr://<letter>/<surname-first>, which is also the
// stable link back to the person's folder. `link` adds both, and writes
// About/<First Surname> (Contacts).webloc opening addressbook://<card id>.
//
// Usage:
//   pplr-contacts check --people-dir DIR [--group NAME] [--json] [--verbose]
//                       [--contacts-json FILE]
//   pplr-contacts link  --people-dir DIR [--group NAME] [--apply]
//                       [--name "Surname, First"]... [--all-names]
//                       [--backup-dir DIR] [--contacts-json FILE]
//   pplr-contacts plan  --people-dir DIR [--group NAME] [--contacts-json FILE]
//   pplr-contacts backup --backup-dir DIR [--contacts-json FILE]
//   pplr-contacts dump [--group NAME]
//
// `plan` proposes, for every pplr person, what should happen to their card:
// Add, Update, Link only, No change or Skip, with the card it would change,
// possible duplicate cards, a confidence, the reason, and the field changes.
// Beyond the strict tiers of `check` it scores near-miss names (Bill and
// William, a hyphenated surname, a one-letter slip), plus a shared company or
// email domain. `pplr sync --plan` renders it as a workbook for review.
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
    var account: String?       // "home" (iCloud, where the group lives), "other", or "none"; absent means home
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
    var aliases: [String] = []  // former keys ("H/Horely, Steve"), from .index/aliases, written by pplr rename
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
            let aliasFile = (pdir as NSString).appendingPathComponent(".index/aliases")
            p.aliases = ((try? String(contentsOfFile: aliasFile, encoding: .utf8)) ?? "")
                .components(separatedBy: .newlines).map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }
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
                        // "[+49 170 0000001](tel:+491700000001)": the link text, not text and URL together;
                        // numbers apart by comma, semicolon, middle dot or slash
                        p.phones = value.components(separatedBy: CharacterSet(charactersIn: ",;·/"))
                            .map { normalisePhone(linkText($0)) }.filter { $0.count > 6 }
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
                          account: acct))
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
    var how: String            // linked | email | linkedin | phone | name
    var inGroup: Bool
    var needsGroup: Bool       // an iCloud card not yet in the group
    var noAccount: Bool        // a directory or Other Known card: nothing can be written
    var hasMarker: Bool        // the card's pplr URL is in the current form
    var markerStale: Bool      // the card has a pplr URL in an older form, to replace
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

/// The path of the card's pplr URL, as written: "b/bray-martin"
func pplrMarker(_ c: Card) -> String? {
    c.urls.first(where: { $0.label.lowercased() == "pplr" || $0.value.hasPrefix("pplr://") })
        .map { $0.value.replacingOccurrences(of: "pplr://", with: "") }
}

/// "B/Bray, Martin" -> "b/bray-martin": lowercase, accents dropped, and
/// every run of other characters a single hyphen, so it is easy to type
func markerPath(_ key: String) -> String {
    let parts = key.split(separator: "/", maxSplits: 1).map(String.init)
    let name = parts.count > 1 ? parts[1] : key
    let folded = name.folding(options: [.caseInsensitive, .diacriticInsensitive], locale: nil).lowercased()
    let slug = folded.unicodeScalars.map { CharacterSet.alphanumerics.contains($0) && $0.isASCII ? String($0) : "-" }.joined()
        .split(separator: "-").joined(separator: "-")
    return "\(parts.count > 1 ? parts[0].lowercased() : String(slug.prefix(1)))/\(slug)"
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
    // Current form ("b/bray-martin") and the first form ("B/Bray,%20Martin")
    var markerKeys: [String: String] = [:]
    // A renamed person answers to their former names too, so an old URL is found and then replaced
    for p in people {
        for k in [p.key] + p.aliases { markerKeys[markerPath(k)] = p.key; markerKeys[k] = p.key }
    }
    var byMarker: [String: Card] = [:], byEmail: [String: [Card]] = [:], bySlug: [String: [Card]] = [:],
        byPhone: [String: [Card]] = [:], byName: [String: [Card]] = [:]
    for c in cards {
        if let m = pplrMarker(c), let key = markerKeys[m] ?? markerKeys[m.removingPercentEncoding ?? m] { byMarker[key] = c }
        for e in c.emails { byEmail[e, default: []].append(c) }
        for s in c.linkedin { bySlug[s, default: []].append(c) }
        for ph in Set(c.phones) where ph.count > 7 { byPhone[ph, default: []].append(c) }
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
                    needsGroup: (c.account ?? "home") == "home" && !c.groups.contains(group),
                    noAccount: c.account == "none",
                    hasMarker: pplrMarker(c) == markerPath(p.key),
                    markerStale: pplrMarker(c) != nil && pplrMarker(c) != markerPath(p.key), webloc: w, hasWebloc: weblocOpens(w, c.id), diffs: diffs(p, c))
    }
    for p in people {
        if let c = byMarker[p.key] { r.linked.append(pair(p, c, "linked")); continue }
        let viaEmail = uniq(p.emails.flatMap { byEmail[$0] ?? [] })
        let viaSlug = p.linkedin.isEmpty ? [] : uniq(bySlug[p.linkedin] ?? [])
        let viaPhone = uniq(p.phones.flatMap { byPhone[$0] ?? [] })
        let viaName = byName[fold("\(p.given) \(p.family)")] ?? []
        if viaEmail.count == 1 { r.matched.append(pair(p, viaEmail[0], "email")) }
        else if viaEmail.count > 1 { r.ambiguous[p.key] = viaEmail.map { "\(displayName($0)) (\($0.id))" } }
        else if viaSlug.count == 1 { r.matched.append(pair(p, viaSlug[0], "linkedin")) }
        else if viaPhone.count == 1 { r.matched.append(pair(p, viaPhone[0], "phone")) }
        else if viaName.count == 1 { r.nameOnly.append(pair(p, viaName[0], "name")) }
        else if viaName.count > 1 { r.ambiguous[p.key] = viaName.map { "\(displayName($0)) (\($0.id))" } }
        else { r.pplrOnly.append(p.key) }
    }
    r.groupOrphans = cards.filter { $0.groups.contains(group) && !claimed.contains($0.id) }.map { displayName($0) }.sorted()
    return r
}

// MARK: - Plan

/// Common English short forms, both ways. Enough to catch Bill for William;
/// anything subtler is for the reviewer.
let nicknames: [String: [String]] = [
    "william": ["bill", "billy", "will", "liam"], "robert": ["rob", "bob", "bobby", "robbie", "bert"],
    "richard": ["rich", "rick", "richie", "dick"], "james": ["jim", "jimmy", "jamie"],
    "alexander": ["alex", "sandy", "xander"], "alexandra": ["alex", "sandra", "lexi"], "alexis": ["alex"],
    "michael": ["mike", "mick", "mikey"], "christopher": ["chris", "kit"], "christine": ["chris", "tina"],
    "christina": ["chris", "tina"], "thomas": ["tom", "tommy"], "anthony": ["tony"], "andrew": ["andy", "drew"],
    "daniel": ["dan", "danny"], "david": ["dave", "davy"], "edward": ["ed", "eddie", "ted", "ned"],
    "elizabeth": ["liz", "beth", "lizzie", "betty", "eliza"], "katherine": ["kate", "kathy", "katie", "kat"],
    "catherine": ["cath", "cathy", "kate", "cat"], "kathryn": ["kate", "kathy", "katie"],
    "jennifer": ["jen", "jenny"], "joseph": ["joe", "joey"], "joshua": ["josh"], "jonathan": ["jon", "jonny", "john"],
    "john": ["jon", "johnny", "jack"], "matthew": ["matt", "matty"], "nicholas": ["nick", "nicky"],
    "patrick": ["pat", "paddy"], "peter": ["pete"], "samuel": ["sam", "sammy"], "samantha": ["sam", "sammy"],
    "stephen": ["steve", "stevie"], "steven": ["steve", "stevie"], "timothy": ["tim", "timmy"],
    "benjamin": ["ben", "benny"], "gregory": ["greg"], "jeffrey": ["jeff"], "geoffrey": ["geoff", "jeff"],
    "kenneth": ["ken", "kenny"], "margaret": ["maggie", "meg", "peggy", "margo"], "rebecca": ["becky", "bec"],
    "susan": ["sue", "suzy"], "suzanne": ["sue", "suzy"], "victoria": ["vicky", "tori"], "zachary": ["zach", "zac"],
    "frederick": ["fred", "freddie"], "gerald": ["gerry"], "lawrence": ["larry", "laurie"], "ronald": ["ron", "ronnie"],
    "donald": ["don", "donny"], "douglas": ["doug"], "philip": ["phil"], "phillip": ["phil"], "raymond": ["ray"],
    "charles": ["charlie", "chuck", "chas"], "henry": ["harry", "hank"], "jacob": ["jake"], "nathan": ["nate", "nat"],
    "nathaniel": ["nate", "nat"], "abigail": ["abby", "abi"], "deborah": ["deb", "debbie"], "pamela": ["pam"],
    "angela": ["angie"], "theodore": ["theo", "ted"], "oliver": ["ollie"], "alfred": ["alf", "alfie"],
    "augustus": ["gus"], "angus": ["gus"], "gustav": ["gus"], "vincent": ["vince", "vinny"], "leonard": ["leo", "len", "lenny"],
    "nicola": ["nicky", "nic"], "nicole": ["nicky", "nic"], "jessica": ["jess", "jessie"], "michelle": ["shelley", "chelle"],
    "francis": ["frank", "fran"], "frances": ["fran", "frankie"], "cameron": ["cam"], "maximilian": ["max"], "maxwell": ["max"],
]

/// Letters and digits only, folded: "O'Brien-Smith" -> "obriensmith"
func squash(_ s: String) -> String {
    String(fold(s).lowercased().unicodeScalars.filter { CharacterSet.alphanumerics.contains($0) }.map(Character.init))
}

/// Name parts, folded: "Dixon-Black" -> ["dixon", "black"]
func tokens(_ s: String) -> [String] {
    fold(s).lowercased().components(separatedBy: CharacterSet.alphanumerics.inverted).filter { !$0.isEmpty }
}

func levenshtein(_ a: String, _ b: String) -> Int {
    let a = Array(a), b = Array(b)
    if a.isEmpty { return b.count }
    if b.isEmpty { return a.count }
    var prev = Array(0...b.count)
    for i in 1...a.count {
        var cur = [i] + Array(repeating: 0, count: b.count)
        for j in 1...b.count {
            cur[j] = min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (a[i - 1] == b[j - 1] ? 0 : 1))
        }
        prev = cur
    }
    return prev[b.count]
}

func isNickname(_ a: String, _ b: String) -> Bool {
    (nicknames[a]?.contains(b) ?? false) || (nicknames[b]?.contains(a) ?? false)
        || nicknames.values.contains { $0.contains(a) && $0.contains(b) }
}

/// Bill and William, Alex and Alexander, Jon and John, Mark and Marc
func givenCompatible(_ a: String, _ b: String) -> Bool {
    let ta = tokens(a), tb = tokens(b)
    guard let x = ta.first, let y = tb.first else { return false }
    if x == y || isNickname(x, y) { return true }
    if min(x.count, y.count) >= 3 && (x.hasPrefix(y) || y.hasPrefix(x)) { return true }
    return min(x.count, y.count) >= 4 && levenshtein(x, y) <= 1
}

/// A shared part of a double-barrelled name, or a one-letter slip
func surnameCompatible(_ a: String, _ b: String) -> Bool {
    let ta = Set(tokens(a).filter { $0.count >= 3 }), tb = Set(tokens(b).filter { $0.count >= 3 })
    if !ta.isDisjoint(with: tb) { return true }
    let x = squash(a), y = squash(b)
    return min(x.count, y.count) >= 5 && levenshtein(x, y) <= 1
}

let genericDomains: Set<String> = ["gmail.com", "googlemail.com", "hotmail.com", "hotmail.co.uk", "outlook.com",
    "live.com", "live.co.uk", "yahoo.com", "yahoo.co.uk", "icloud.com", "me.com", "mac.com", "aol.com",
    "btinternet.com", "protonmail.com", "proton.me", "msn.com", "fastmail.com", "fastmail.fm"]

func domains(_ emails: [String]) -> Set<String> {
    Set(emails.compactMap { $0.split(separator: "@").last.map(String.init) }).subtracting(genericDomains)
}

struct Candidate: Codable {
    var ref: String
    var name: String
    var score: Int
    var why: String
}

/// How near a card is to a person by name, company and email domain; nil when
/// the names have nothing in common
func score(_ p: Person, _ c: Card) -> (Int, String)? {
    let pg = squash(p.given), pf = squash(p.family), cg = squash(c.given), cf = squash(c.family)
    guard !cg.isEmpty || !cf.isEmpty else { return nil }
    var s: Int, why: [String]
    if pg == cg && pf == cf { s = 60; why = ["same name"] }
    else if pg == cf && pf == cg { s = 45; why = ["name reversed"] }
    else if pf == cf && givenCompatible(p.given, c.given) { s = 45; why = ["same surname, \(c.given) for \(p.given)"] }
    else if pg == cg && surnameCompatible(p.family, c.family) { s = 40; why = ["same given name, \(c.family) for \(p.family)"] }
    else if givenCompatible(p.given, c.given) && surnameCompatible(p.family, c.family) { s = 25; why = ["similar name"] }
    else { return nil }
    let pc = squash(p.company), cc = squash(c.organization)
    if pc.count >= 3 && cc.count >= 3 && (pc.contains(cc) || cc.contains(pc)) { s += 20; why.append("same company") }
    if !domains(p.emails).isDisjoint(with: domains(c.emails)) { s += 15; why.append("same email domain") }
    return (s, why.joined(separator: ", "))
}

struct PlanRow: Codable {
    var person: String
    var decision: String        // Add | Update | Link only | No change | Skip
    var confidence: String      // High | Medium | Low
    var why: String
    var card: String?           // the card's ref, eg C0042
    var dupes: [String]         // other cards that look like the same person
    var changes: [String]
    var candidates: [Candidate] // the other near cards, best first
}

struct PlanCard: Codable {
    var ref: String
    var id: String
    var name: String
    var given: String
    var family: String
    var organization: String
    var jobTitle: String
    var emails: [String]
    var phones: [String]
    var linkedin: [String]
    var account: String
    var groups: [String]
    var marker: String?
    var proposedFor: String?    // the pplr person the plan would give this card to
}

struct PlanPerson: Codable {
    var key: String
    var given: String
    var family: String
    var company: String
    var role: String
    var emails: [String]
    var phones: [String]
    var linkedin: String
    var marker: String
    var picture: String?        // About/<First Surname> (Picture).jpg, if there is one
    var dir: String
}

struct Plan: Codable {
    var generated: String
    var group: String
    var rows: [PlanRow]
    var people: [PlanPerson]
    var cards: [PlanCard]
}

func picturePath(_ p: Person) -> String? {
    let about = (p.personDir as NSString).appendingPathComponent("About")
    return ((try? FileManager.default.contentsOfDirectory(atPath: about)) ?? []).sorted()
        .first(where: { $0.hasSuffix("(Picture).jpg") || $0.hasSuffix("(Picture).png") || $0.hasSuffix("(Picture).jpeg") })
        .map { (about as NSString).appendingPathComponent($0) }
}

/// What Update would do to the card: pplr wins on name, company and role;
/// emails, phones and LinkedIn are added; nothing on the card is removed
func fieldChanges(_ p: Person, _ c: Card) -> [String] {
    var out: [String] = []
    let pname = "\(p.given) \(p.family)", cname = [c.given, c.family].filter { !$0.isEmpty }.joined(separator: " ")
    if fold(pname) != fold(cname) { out.append("name: \(pname) (was \(cname.isEmpty ? "empty" : cname))") }
    if !p.company.isEmpty && fold(p.company) != fold(c.organization) {
        out.append("company: \(p.company) (was \(c.organization.isEmpty ? "empty" : c.organization))")
    }
    if !p.role.isEmpty && fold(p.role) != fold(c.jobTitle) {
        out.append("role: \(p.role) (was \(c.jobTitle.isEmpty ? "empty" : c.jobTitle))")
    }
    for e in p.emails where !c.emails.contains(e) { out.append("+ email \(e)") }
    for ph in p.phones where !c.phones.contains(ph) { out.append("+ phone \(ph)") }
    if !p.linkedin.isEmpty && !c.linkedin.contains(p.linkedin) { out.append("+ LinkedIn \(p.linkedin)") }
    return out
}

func linkChanges(_ p: Person, _ c: Card, group: String) -> [String] {
    var out: [String] = []
    if let m = pplrMarker(c), m != markerPath(p.key) { out.append("pplr URL updated") }
    else if pplrMarker(c) == nil { out.append("+ pplr URL") }
    if (c.account ?? "home") == "home" && !c.groups.contains(group) { out.append("+ \(group) group") }
    return out
}

func addChanges(_ p: Person, group: String) -> [String] {
    var parts = ["name"]
    if !p.company.isEmpty { parts.append("company") }
    if !p.role.isEmpty { parts.append("role") }
    if !p.emails.isEmpty { parts.append(p.emails.count == 1 ? "email" : "\(p.emails.count) emails") }
    if !p.phones.isEmpty { parts.append(p.phones.count == 1 ? "phone" : "\(p.phones.count) phones") }
    if !p.linkedin.isEmpty { parts.append("LinkedIn") }
    if picturePath(p) != nil { parts.append("photo") }
    return ["new iCloud card: " + parts.joined(separator: ", "), "+ pplr URL", "+ \(group) group"]
}

func plan(people: [Person], cards rawCards: [Card], group: String) -> Plan {
    // One entry per real card, each with a short ref for the reviewer
    var seen = Set<String>()
    let cards = rawCards.filter { seen.insert($0.id).inserted }
        .sorted { (fold($0.family), fold($0.given), $0.organization, $0.id) < (fold($1.family), fold($1.given), $1.organization, $1.id) }
    var refOf: [String: String] = [:]
    for (i, c) in cards.enumerated() { refOf[c.id] = String(format: "C%04d", i + 1) }
    let cardByID = Dictionary(uniqueKeysWithValues: cards.map { ($0.id, $0) })
    let byKey = Dictionary(uniqueKeysWithValues: people.map { ($0.key, $0) })
    let report = check(people: people, cards: cards, group: group)

    var rows: [PlanRow] = []
    var done = Set<String>()
    // The strict tiers: the pplr URL, then a shared email, LinkedIn or phone
    let strong = Set((report.linked + report.matched).map(\.contact))
    let tier = ["linked": "linked by pplr URL", "email": "same email", "linkedin": "same LinkedIn", "phone": "same phone"]
    for pr in report.linked + report.matched {
        let p = byKey[pr.person]!, c = cardByID[pr.contact]!
        done.insert(p.key)
        if pr.noAccount {
            rows.append(PlanRow(person: p.key, decision: "Add", confidence: "Medium",
                                why: "\(tier[pr.how]!), but that card is in no account (Other Known or a directory)",
                                card: nil, dupes: [], changes: addChanges(p, group: group),
                                candidates: [Candidate(ref: refOf[c.id]!, name: displayName(c), score: 100, why: tier[pr.how]!)]))
            continue
        }
        let fields = fieldChanges(p, c), link = linkChanges(p, c, group: group)
        let decision = !fields.isEmpty ? "Update" : !link.isEmpty ? "Link only" : "No change"
        // A rename is the reviewer's call: pplr's own spelling can be the wrong one
        let renames = fields.contains { $0.hasPrefix("name: ") }
        rows.append(PlanRow(person: p.key, decision: decision, confidence: renames ? "Medium" : "High",
                            why: tier[pr.how]! + (renames ? ", names differ" : ""),
                            card: refOf[c.id], dupes: [], changes: fields + link, candidates: []))
    }
    // Everyone else: score the cards the strict tiers left free. A shared
    // email (several cards carrying it) outranks any name.
    let free = cards.filter { !strong.contains($0.id) }
    typealias Scored = (card: Card, score: Int, why: String)
    for p in people where !done.contains(p.key) {
        let mine = Set(p.emails)
        var cands: [Scored] = free.compactMap { c in
            let shares = c.emails.contains(where: mine.contains)
            if let (s, why) = score(p, c) { return (c, s + (shares ? 40 : 0), why + (shares ? ", same email" : "")) }
            return shares ? (c, 40, "same email, different name") : nil
        }
        let home = { (c: Scored) in (c.card.account ?? "home") == "home" ? 1 : 0 }
        cands.sort { ($0.score, home($0)) > ($1.score, home($1)) }
        let asCand = { (c: Scored) in Candidate(ref: refOf[c.card.id]!, name: displayName(c.card), score: c.score, why: c.why) }
        guard let best = cands.first, best.score >= 40 else {
            rows.append(PlanRow(person: p.key, decision: "Add", confidence: cands.isEmpty ? "High" : "Low",
                                why: cands.isEmpty ? "no card found" : "only weak candidates",
                                card: nil, dupes: [], changes: addChanges(p, group: group), candidates: cands.map(asCand)))
            continue
        }
        if best.card.account == "none" {
            rows.append(PlanRow(person: p.key, decision: "Add", confidence: "Low",
                                why: "\(best.why), but that card is in no account (Other Known or a directory)",
                                card: nil, dupes: [], changes: addChanges(p, group: group), candidates: cands.map(asCand)))
            continue
        }
        // Cards nearly as good as the best look like duplicates of one person
        let twins = cands.dropFirst().filter { $0.score >= 60 && $0.score >= best.score - 20 && $0.card.account != "none" }
        let twinIDs = Set(twins.map(\.card.id))
        let fields = fieldChanges(p, best.card), link = linkChanges(p, best.card, group: group)
        let renames = fields.contains { $0.hasPrefix("name: ") }
        let confidence = !twins.isEmpty ? "Low" : best.score >= 80 && !renames ? "High" : best.score >= 60 ? "Medium" : "Low"
        rows.append(PlanRow(person: p.key, decision: fields.isEmpty ? "Link only" : "Update", confidence: confidence,
                            why: best.why + (twins.isEmpty ? "" : "; \(twins.count + 1) cards look like this person"),
                            card: refOf[best.card.id], dupes: twins.map { refOf[$0.card.id]! }, changes: fields + link,
                            candidates: cands.dropFirst().filter { !twinIDs.contains($0.card.id) }.prefix(3).map(asCand)))
    }
    // One card proposed for two people: neither can be trusted
    let claims = Dictionary(grouping: rows.filter { $0.card != nil }, by: { $0.card! })
    for i in rows.indices {
        guard let ref = rows[i].card, let others = claims[ref], others.count > 1 else { continue }
        rows[i].confidence = "Low"
        rows[i].why += "; card also proposed for " + others.map(\.person).filter { $0 != rows[i].person }.joined(separator: ", ")
    }
    let rank = ["Low": 0, "Medium": 1, "High": 2], drank = ["Update": 0, "Link only": 1, "Add": 2, "No change": 3, "Skip": 4]
    rows.sort { (rank[$0.confidence]!, drank[$0.decision]!, $0.person) < (rank[$1.confidence]!, drank[$1.decision]!, $1.person) }

    var proposed: [String: String] = [:]
    for r in rows {
        if let c = r.card { proposed[c] = r.person }
        for d in r.dupes { proposed[d] = "dupe? " + r.person }
    }
    let f = DateFormatter(); f.dateFormat = "yyyy-MM-dd HH:mm"
    return Plan(generated: f.string(from: Date()), group: group, rows: rows,
                people: people.map { p in
                    PlanPerson(key: p.key, given: p.given, family: p.family, company: p.company, role: p.role,
                               emails: p.emails, phones: p.phones, linkedin: p.linkedin, marker: markerURL(p.key),
                               picture: picturePath(p), dir: p.personDir)
                },
                cards: cards.map { c in
                    PlanCard(ref: refOf[c.id]!, id: c.id, name: displayName(c), given: c.given, family: c.family,
                             organization: c.organization, jobTitle: c.jobTitle, emails: c.emails, phones: c.phones,
                             linkedin: c.linkedin, account: c.account ?? "home", groups: c.groups,
                             marker: pplrMarker(c), proposedFor: proposed[refOf[c.id]!])
                })
}

// MARK: - pplr:// URLs

/// pplr://<letter>/<surname-first>[/<path inside the person's folder>]. The
/// person part is their marker (or a former one, from their aliases); a bare
/// URL means the person, and opens their About file.
struct PplrTarget {
    var person: Person
    var rest: String            // "" for the person, else eg "Meetings/20260828 Catch-up/Notes.md"
    var path: String            // the file or folder it names
}

func peopleByMarker(_ people: [Person]) -> [String: Person] {
    var out: [String: Person] = [:]
    for p in people { for k in [p.key] + p.aliases { out[markerPath(k)] = p } }
    return out
}

func peopleByKey(_ people: [Person]) -> [String: Person] {
    var out: [String: Person] = [:]
    for p in people { for k in [p.key] + p.aliases { out[k] = p } }
    return out
}

func personPage(_ p: Person) -> String { p.aboutPath.isEmpty ? p.personDir : p.aboutPath }

func resolvePplr(_ url: String, _ byMarker: [String: Person]) -> PplrTarget? {
    guard url.lowercased().hasPrefix("pplr://") else { return nil }
    let body = String(url.dropFirst("pplr://".count))
    let parts = body.split(separator: "/", maxSplits: 2, omittingEmptySubsequences: false).map(String.init)
    guard parts.count >= 2, let p = byMarker["\(parts[0].lowercased())/\(parts[1].lowercased())"] else { return nil }
    let rest = parts.count > 2 ? (parts[2].removingPercentEncoding ?? parts[2]) : ""
    let path = rest.isEmpty ? personPage(p) : (p.personDir as NSString).appendingPathComponent(rest)
    return PplrTarget(person: p, rest: rest, path: path)
}

/// A link into the people tree, as a file path ("../../Career/People/K/Kemp, Jon/About/...")
/// or a CMS URL ("http://localhost:4360/people/K/Kemp%2C%20Jon/..."): the key and the rest
func peopleLinkParts(_ target: String) -> (key: String, rest: String)? {
    var sub: String
    if let r = target.range(of: "Career/People/") { sub = String(target[r.upperBound...]) }
    else if let r = target.range(of: "localhost:4360/people/") { sub = String(target[r.upperBound...]) }
    else { return nil }
    if let q = sub.firstIndex(where: { $0 == "?" || $0 == "#" }) { sub = String(sub[..<q]) }
    sub = sub.removingPercentEncoding ?? sub
    let parts = sub.split(separator: "/", maxSplits: 2, omittingEmptySubsequences: false).map(String.init)
    guard parts.count >= 2, parts[0].count == 1, parts[1].contains(", ") else { return nil }
    return ("\(parts[0])/\(parts[1])", parts.count > 2 ? parts[2] : "")
}

struct LinkChange: Codable {
    var file: String
    var line: Int
    var from: String
    var to: String?             // nil: left as it was
    var problem: String?        // why it was left: no such person, or no such file
}

/// Rewrites Markdown links into the people tree as pplr:// URLs. A link to a
/// person's About file (or folder) becomes the bare person URL; a link to
/// anything else keeps its path inside the folder. Links whose person or file
/// cannot be found are left as they are and reported.
func convertLinks(in text: String, file: String, _ byKey: [String: Person]) -> (String, [LinkChange]) {
    // ](<target>) and ](target)
    let re = try! NSRegularExpression(pattern: #"\]\(\s*(?:<([^>\n]+)>|([^)\s]+))\s*\)"#)
    let ns = text as NSString
    var out = text as NSString, changes: [LinkChange] = []
    for m in re.matches(in: text, range: NSRange(location: 0, length: ns.length)).reversed() {
        let g = m.range(at: 1).location != NSNotFound ? m.range(at: 1) : m.range(at: 2)
        let target = ns.substring(with: g)
        guard let (key, rawRest) = peopleLinkParts(target) else { continue }
        let line = ns.substring(to: m.range.location).components(separatedBy: "\n").count
        guard let p = byKey[key] else {
            changes.append(LinkChange(file: file, line: line, from: target, to: nil, problem: "no such person: \(key)"))
            continue
        }
        var rest = rawRest.hasSuffix("/") ? String(rawRest.dropLast()) : rawRest
        // The About file is the person: its name follows a rename, the person URL does not
        if (rest.hasPrefix("About/") && rest.hasSuffix("(About).md")) || rest == "About" { rest = "" }
        if !rest.isEmpty && !FileManager.default.fileExists(atPath: (p.personDir as NSString).appendingPathComponent(rest)) {
            changes.append(LinkChange(file: file, line: line, from: target, to: nil, problem: "no such file in \(p.key): \(rest)"))
            continue
        }
        let url = markerURL(p.key) + (rest.isEmpty ? "" : "/" + rest)
        let link = url.contains(where: { " ()<>".contains($0) }) ? "](<\(url)>)" : "](\(url))"
        out = out.replacingCharacters(in: m.range, with: link) as NSString
        changes.append(LinkChange(file: file, line: line, from: target, to: url, problem: nil))
    }
    return (out as String, changes.reversed())
}

/// Each real file once: a symlinked note (eg day_notes.md -> this month's) is
/// followed to its target, so it is neither counted twice nor replaced by a
/// plain file when written
func markdownFiles(_ roots: [String]) -> [String] {
    var seen = Set<String>()
    return rawMarkdownFiles(roots).map { URL(fileURLWithPath: $0).resolvingSymlinksInPath().path }
        .filter { seen.insert($0).inserted }.sorted()
}

func rawMarkdownFiles(_ roots: [String]) -> [String] {
    var out: [String] = []
    let fm = FileManager.default
    for r in roots {
        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: r, isDirectory: &isDir) else { continue }
        if !isDir.boolValue { out.append(r); continue }
        let base = URL(fileURLWithPath: r).resolvingSymlinksInPath()
        let e = fm.enumerator(at: base, includingPropertiesForKeys: nil, options: [.skipsHiddenFiles])
        while let u = e?.nextObject() as? URL {
            if u.pathExtension == "md" { out.append(u.path) }
        }
    }
    return out.sorted()
}

// MARK: - Link

/// pplr://k/kemp-jon
func markerURL(_ key: String) -> String { "pplr://" + markerPath(key) }

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
    var noAccount: [String]     // matched, but the card cannot be written
}

func planLink(_ r: Report, names: [String], allNames: Bool) -> LinkPlan {
    let wanted = Set(names.map(personKey))
    let nameKeys = Set(r.nameOnly.map(\.person))
    let confirmed = r.nameOnly.filter { allNames || wanted.contains($0.person) }
    let all = r.linked + r.matched + confirmed
    let done = { (p: Pair) in p.hasMarker && !p.needsGroup && p.hasWebloc }
    return LinkPlan(toLink: all.filter { !done($0) && !$0.noAccount },
                    alreadyDone: all.filter(done).count,
                    namesSkipped: r.nameOnly.filter { !(allNames || wanted.contains($0.person)) }.map(\.person),
                    unknownNames: wanted.subtracting(nameKeys).subtracting((r.linked + r.matched).map(\.person)).sorted(),
                    ambiguous: r.ambiguous.keys.sorted(),
                    noAccount: all.filter(\.noAccount).map(\.person))
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

func applyFixture(_ plan: LinkPlan, group: String, fixture: String, limit: Int?) throws -> Outcome {
    var cards = try JSONDecoder().decode([Card].self, from: Data(contentsOf: URL(fileURLWithPath: fixture)))
    var out: Outcome = []
    for p in plan.toLink.prefix(limit ?? Int.max) {
        guard let i = cards.firstIndex(where: { $0.id == p.contact }) else { continue }
        if p.markerStale, let u = cards[i].urls.firstIndex(where: { $0.label.lowercased() == "pplr" || $0.value.hasPrefix("pplr://") }) {
            cards[i].urls[u].value = markerURL(p.person)
        } else if !p.hasMarker { cards[i].urls.append(LabelledValue(label: "pplr", value: markerURL(p.person))) }
        if p.needsGroup { cards[i].groups.append(group) }
        if !p.hasWebloc { try writeWebloc(p.webloc, p.contact) }
        out.append((p.person, nil))
    }
    let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
    try enc.encode(cards).write(to: URL(fileURLWithPath: fixture))
    return out
}

/// Contacts.app makes the card changes. The Contacts framework refused to
/// update a card from this tool (CoreData, Cocoa error 134092) although it
/// could create a group; the app has full access and takes the same ids.
let linkScript = """
on run argv
    set pid to item 1 of argv
    set marker to item 2 of argv
    set gid to item 3 of argv
    set replacing to item 4 of argv
    tell application "Contacts"
        set p to person id pid
        if marker is not "" then
            if replacing is "yes" then
                set value of (first url of p whose label is "pplr") to marker
            else
                make new url at end of urls of p with properties {label:"pplr", value:marker}
            end if
        end if
        if gid is not "" then add p to group id gid
        save
    end tell
end run
"""

func osascript(_ args: [String]) -> String? {
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
    proc.arguments = ["-e", linkScript] + args
    let err = Pipe(); proc.standardError = err; proc.standardOutput = Pipe()
    do { try proc.run() } catch { return error.localizedDescription }
    proc.waitUntilExit()
    if proc.terminationStatus == 0 { return nil }
    let msg = String(data: err.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
    return msg.trimmingCharacters(in: .whitespacesAndNewlines)
}

/// person -> nil when linked, or the error
typealias Outcome = [(person: String, error: String?)]

func applyLive(_ plan: LinkPlan, group: String, limit: Int?) throws -> Outcome {
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
    var out: Outcome = []
    var failures = 0
    for p in plan.toLink.prefix(limit ?? Int.max) {
        var error: String? = nil
        do {
            let gid = p.needsGroup ? try groupIn(store.defaultContainerIdentifier()).identifier : ""
            if !p.hasMarker || p.needsGroup {
                error = osascript([p.contact, p.hasMarker ? "" : markerURL(p.person), gid, p.markerStale ? "yes" : "no"])
            }
            if error == nil && !p.hasWebloc { try writeWebloc(p.webloc, p.contact) }
        } catch let e { error = e.localizedDescription }
        out.append((p.person, error))
        if error != nil { failures += 1; if failures >= 3 { break } }
    }
    return out
}


func printPlan(_ plan: LinkPlan, group: String, outcome: Outcome?, backupPath: String?) {
    func what(_ p: Pair) -> String {
        var w: [String] = []
        if p.markerStale { w.append("pplr URL updated") } else if !p.hasMarker { w.append("pplr URL") }
        if p.needsGroup { w.append("group") }
        if !p.hasWebloc { w.append(".webloc") }
        return "[\(p.how); \(w.joined(separator: " + "))]"
    }
    let byPerson = Dictionary(uniqueKeysWithValues: plan.toLink.map { ($0.person, $0) })
    if let outcome {
        let ok = outcome.filter { $0.error == nil }, bad = outcome.filter { $0.error != nil }
        print("pplr sync --link --apply")
        print("")
        print("  linked now              \(ok.count)")
        print("  failed                  \(bad.count)")
        print("  still to link           \(plan.toLink.count - ok.count)")
        print("  already linked          \(plan.alreadyDone)")
        if !ok.isEmpty {
            print("\nLinked (the pplr URL on the card, the \"\(group)\" group for iCloud cards; a (Contacts).webloc in About):")
            ok.forEach { o in print("  \(o.person)  <->  \(byPerson[o.person]!.contactName)  \(what(byPerson[o.person]!))") }
        }
        if !bad.isEmpty {
            print("\nFailed:")
            bad.forEach { print("  \($0.person): \($0.error!)") }
            if bad.count >= 3 { print("  (stopped after three failures)") }
        }
    } else {
        print("pplr sync --link (dry run: nothing written; --apply to write)")
        print("")
        print("  to link                 \(plan.toLink.count)")
        print("  already linked          \(plan.alreadyDone)")
        print("  name only, not linked   \(plan.namesSkipped.count)")
        print("  ambiguous, not linked   \(plan.ambiguous.count)")
        if !plan.toLink.isEmpty {
            print("\nWould link (the pplr URL on the card, the \"\(group)\" group for iCloud cards, nothing else; a (Contacts).webloc in About):")
            plan.toLink.forEach { print("  \($0.person)  <->  \($0.contactName)  \(what($0))") }
        }
    }
    if !plan.noAccount.isEmpty {
        print("\nMatched, but the card is in no account (a directory or Other Known card?), so cannot be linked:")
        plan.noAccount.forEach { print("  \($0)") }
    }
    if !plan.unknownNames.isEmpty {
        print("\nNot a name-only match, so ignored:")
        plan.unknownNames.forEach { print("  \($0)") }
    }
    if outcome == nil && !plan.namesSkipped.isEmpty {
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
    print("  matched by phone        \(r.matched.filter { $0.how == "phone" }.count)")
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
    section("Name only: same name, no shared email, LinkedIn or phone. Confirm before linking:",
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
    case "plan":
        guard let peopleDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "plan needs --people-dir"]) }
        let result = plan(people: loadPeople(peopleDir), cards: try loadCards(fixture: fixture), group: group)
        let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        print(String(data: try enc.encode(result), encoding: .utf8)!)
    case "resolve":
        guard let peopleDir, let url = args.first else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "resolve needs --people-dir and a pplr:// URL"]) }
        guard let t = resolvePplr(url, peopleByMarker(loadPeople(peopleDir))) else {
            FileHandle.standardError.write(Data("pplr: no one answers to \(url)\n".utf8)); exit(4)
        }
        print(t.path)
    case "links":
        guard let peopleDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "links needs --people-dir"]) }
        let byKey = peopleByKey(loadPeople(peopleDir))
        var all: [LinkChange] = []
        for f in markdownFiles(args) {
            guard let text = try? String(contentsOfFile: f, encoding: .utf8) else { continue }
            let (out, changes) = convertLinks(in: text, file: f, byKey)
            all += changes
            if apply && out != text { try out.write(toFile: f, atomically: true, encoding: .utf8) }
        }
        if asJSON {
            let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
            print(String(data: try enc.encode(all), encoding: .utf8)!)
        } else {
            let done = all.filter { $0.to != nil }, left = all.filter { $0.to == nil }
            print("pplr links\(apply ? "" : " (dry run: nothing written; --apply to write)")\n")
            print("  links into the people tree  \(all.count)")
            print("  \((apply ? "converted" : "to convert").padding(toLength: 28, withPad: " ", startingAt: 0))\(done.count)")
            print("  left as they are            \(left.count)")
            print("  files                       \(Set(done.map(\.file)).count)")
            if !left.isEmpty {
                print("\nLeft as they are:")
                for c in left { print("  \(c.file):\(c.line)  \(c.problem!)") }
            }
        }
    case "backup":
        guard let backupDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "backup needs --backup-dir"]) }
        print("Backup: \(try backup(to: backupDir, fixture: fixture))")
    case "link":
        guard let peopleDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "link needs --people-dir"]) }
        let report = check(people: loadPeople(peopleDir), cards: try loadCards(fixture: fixture), group: group)
        let plan = planLink(report, names: names, allNames: allNames)
        var backupPath: String? = nil
        var outcome: Outcome? = nil
        if apply {
            outcome = []
            if !plan.toLink.isEmpty {
                guard let backupDir else { throw NSError(domain: "pplr", code: 2, userInfo: [NSLocalizedDescriptionKey: "--apply needs --backup-dir"]) }
                backupPath = try backup(to: backupDir, fixture: fixture)
                outcome = fixture != nil
                    ? try applyFixture(plan, group: group, fixture: fixture!, limit: limit)
                    : try applyLive(plan, group: group, limit: limit)
            }
        }
        printPlan(plan, group: group, outcome: outcome, backupPath: backupPath)
        if outcome?.contains(where: { $0.error != nil }) == true { exit(1) }
    default:
        FileHandle.standardError.write(Data("usage: pplr-contacts check --people-dir DIR [--group NAME] [--json] [--verbose] [--contacts-json FILE]\n       pplr-contacts plan --people-dir DIR [--group NAME] [--contacts-json FILE]\n       pplr-contacts link --people-dir DIR [--group NAME] [--apply] [--name NAME]... [--all-names] [--backup-dir DIR]\n       pplr-contacts dump [--group NAME] [--only-group]\n".utf8))
        exit(2)
    }
} catch {
    FileHandle.standardError.write(Data("pplr-contacts: \(error.localizedDescription)\n".utf8))
    exit(1)
}
