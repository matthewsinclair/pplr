# packaging/homebrew/pplr.rb: the template of pplr's Homebrew formula
# (ST0002 WP04, design E1 and T8). bin/devbin formula render fills it from a
# release: @@VERSION@@, each asset's @@SHA256:...@@ from the release's
# SHA256SUMS, @@INSTALL@@ (the assets into libexec, 0755 on bin/pplr, the T3
# manifest with managed_by: brew, the link in bin). This header is dropped
# from the render. Edit this file, never a rendered one.
#
# REQUIREMENTS repeats packaging/macos/signing's designated requirements,
# which a Ruby formula cannot read; a cargo test fails when the two differ.
#
# AT-04.1 (ST0002 AC-04.1): rendered with nothing typed, the formula installs
# through @@INSTALL@@ and verify_signatures, which refuses a binary that does not
# meet its requirement, and its test re-verifies and runs the version line.
class Pplr < Formula
  desc "Personal relationship manager over a plain-text People tree"
  homepage "https://github.com/matthewsinclair/pplr"
  url "https://github.com/matthewsinclair/pplr/releases/download/v@@VERSION@@/pplr-@@VERSION@@-aarch64-apple-darwin.tar.gz"
  sha256 "@@SHA256:pplr-{version}-aarch64-apple-darwin.tar.gz@@"
  license "MIT"

  # macOS on Apple silicon alone, as Intent's Mac target (hv, 2026-11-02). The
  # Linux tarballs join it in a later release.
  depends_on arch: :arm64
  depends_on :macos

  # The designated requirement each macOS binary is signed to: its identifier,
  # then a Developer ID certificate for Geodica's team
  DEVELOPER_ID = "anchor apple generic and certificate 1[field.1.2.840.113635.100.6.2.6] " \
                 "and certificate leaf[field.1.2.840.113635.100.6.1.13] " \
                 "and certificate leaf[subject.OU] = \"76BQL8L47U\"".freeze
  REQUIREMENTS = {
    "pplr"           => "identifier \"com.geodica.pplr\" and #{DEVELOPER_ID}",
    "pplr-contacts"  => "identifier \"com.geodica.pplr-contacts\" and #{DEVELOPER_ID}",
    "Pplr Links.app" => "identifier \"com.geodica.pplr-links\" and #{DEVELOPER_ID}",
  }.freeze

  def verify_signatures
    REQUIREMENTS.each do |binary, requirement|
      # The binaries are in bin/, the pplr:// handler app at the root
      path = binary.end_with?(".app") ? libexec/binary : libexec/"bin"/binary
      system "/usr/bin/codesign", "--verify", "--strict", "-R=#{requirement}", path
    end
  end

  def install
    @@INSTALL@@
    verify_signatures if OS.mac?
  end

  def caveats
    <<~EOS
      pplr reads your People tree from $PPLR_DATA (default ~/Dropbox/Career/People).

      If you also run pplr from a checkout, `pplr use brew` hands the pplr
      command back to Homebrew.

      On macOS, the first `pplr sync` asks for access to Contacts for
      pplr-contacts; allow it in System Settings > Privacy & Security > Contacts.
    EOS
  end

  test do
    verify_signatures if OS.mac?
    assert_match "pplr #{version} (brew, ", shell_output("#{bin}/pplr --version")
  end
end
