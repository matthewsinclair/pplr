// The version line, devbin's docs/dev-opt-brew.md T4 (devbin#0326,
// pplr#0001): in an opt or brew tree, the `line` its manifest records;
// otherwise the line bin/devbin build stamped; either under this binary's own
// name. The contract's reader, as src/main.rs carries it for pplr, in Swift for
// the second binary of pplr's tree. bin/devbin install proves each binary in
// bin/ by its --version before it publishes it.
//
// `versionStamp` is the stamped line, which bin/.devbin/cmd/build writes into
// target/<profile>/pplr-contacts-stamp.swift from DEVBIN_VERSION_LINE, or as
// `pplr <version> (unknown, unknown)` when the build was not handed one.

import Foundation

func versionLine() -> String {
    let tool = "pplr"
    let name = "pplr-contacts"
    let manifest = ".pplr-install"
    func named(_ line: String) -> String {
        guard let space = line.firstIndex(of: " ") else { return line }
        return name + line[space...]
    }
    if let exe = Bundle.main.executableURL?.resolvingSymlinksInPath() {
        let root = exe.deletingLastPathComponent().deletingLastPathComponent()
        if let text = try? String(contentsOf: root.appendingPathComponent(manifest), encoding: .utf8) {
            let lines = text.components(separatedBy: "\n")
            if lines.contains("# tool: \(tool)"),
               let line = lines.first(where: { $0.hasPrefix("# line: ") }) {
                return named(String(line.dropFirst("# line: ".count)))
            }
        }
    }
    return named(versionStamp)
}
