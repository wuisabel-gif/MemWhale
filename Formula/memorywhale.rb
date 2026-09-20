# Homebrew formula for the MemoryWhale CLI.
#
# Use as a tap:
#   brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
#   brew install memorywhale
#
# Maintainer note: `url`/`sha256` are updated automatically by the
# bump-formula job in .github/workflows/release.yml on every tagged release.
class Memorywhale < Formula
  desc "Local-first terminal memory: record commands, sessions, and output into SQLite"
  homepage "https://github.com/wuisabel-gif/MemWhale"
  url "https://github.com/wuisabel-gif/MemWhale/archive/refs/tags/v0.11.1.tar.gz"
  sha256 "e38d129387c7f86a6c1dadbc17ee4ac384127121f7af4b740386e6a171eede5d"
  license "MIT"
  head "https://github.com/wuisabel-gif/MemWhale.git", branch: "main"

  depends_on "rust" => :build

  def install
    # Build only the dependency-light CLI crate (no Tauri/GTK).
    system "cargo", "install", "--locked", "--path", "crates/mw-cli", "--root", prefix
  end

  test do
    assert_equal "mw #{version}", shell_output("#{bin}/mw --version").strip
    assert_match "record a whole shell session", shell_output("#{bin}/mw --help")
    ENV["MEMORYWHALE_DATA_DIR"] = (testpath/"data").to_s
    system bin/"mw-remember", "--cwd", testpath, "--exit-code", "0", "--", "brew-test"
    assert_match "brew-test", shell_output("#{bin}/mw search brew-test agent:terminal")
  end
end
