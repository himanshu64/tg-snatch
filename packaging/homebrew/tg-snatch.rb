class TgSnatch < Formula
  desc "Snatch files from Telegram — fast, secure, beautiful CLI downloader"
  homepage "https://github.com/himanshu64/tg-snatch"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/himanshu64/tg-snatch/releases/download/v#{version}/tg-snatch-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X86"
    end

    on_arm do
      url "https://github.com/himanshu64/tg-snatch/releases/download/v#{version}/tg-snatch-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/himanshu64/tg-snatch/releases/download/v#{version}/tg-snatch-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86"
    end

    on_arm do
      url "https://github.com/himanshu64/tg-snatch/releases/download/v#{version}/tg-snatch-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM"
    end
  end

  depends_on "curl"

  def install
    bin.install "tg-snatch"
  end

  test do
    assert_match "tg-snatch", shell_output("#{bin}/tg-snatch --version")
  end
end
