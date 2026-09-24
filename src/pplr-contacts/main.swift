// pplr-contacts: the Apple Contacts engine behind `pplr sync`.
//
// Read-only for now. `check` compares the pplr people tree with Contacts and
// reports who is linked, who matches, who is missing on each side, and which
// schema fields differ. Only the header fields of each About file take part:
// name, Role, Company, Email, Phone and LinkedIn. Notes, bios and meetings
// never leave pplr.
//
// Provenance in Contacts (for the later push): membership of the "PPLR" group,
// plus a URL labelled "pplr" whose value is pplr://<Letter>/<Surname, First>,
// which is also the stable link back to the person's folder.
//
// Usage:
//   pplr-contacts check --people-dir DIR [--group NAME] [--json] [--verbose]
//                       [--contacts-json FILE]
//   pplr-contacts dump [--group NAME]
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
                           family: parts[0], role: "", company: "", emails: [], phones: [], linkedin: "", aboutPath: "")
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

func loadCards(fixture: String?) throws -> [Card] {
    if let fixture {
        return try JSONDecoder().decode([Card].self, from: Data(contentsOf: URL(fileURLWithPath: fixture)))
    }
    let store = CNContactStore()
    guard requestAccess(store) else {
        FileHandle.standardError.write(Data("""
        pplr sync: no access to Contacts.
        Allow it in System Settings > Privacy & Security > Contacts for the app running pplr (eg iTerm or Terminal), then run it again.\n
        """.utf8))
        exit(3)
    }
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
        return Pair(person: p.key, contact: c.id, contactName: displayName(c), how: how, inGroup: c.groups.contains(group), diffs: diffs(p, c))
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
    default:
        FileHandle.standardError.write(Data("usage: pplr-contacts check --people-dir DIR [--group NAME] [--json] [--verbose] [--contacts-json FILE]\n       pplr-contacts dump [--group NAME] [--only-group]\n".utf8))
        exit(2)
    }
} catch {
    FileHandle.standardError.write(Data("pplr-contacts: \(error.localizedDescription)\n".utf8))
    exit(1)
}
