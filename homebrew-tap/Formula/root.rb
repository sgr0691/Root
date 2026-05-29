class Root < Formula
  desc "Deterministic package manager powered by Nix"
  homepage "https://github.com/sgr0691/Root"
  version "0.1.0"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/sgr0691/Root/releases/download/v#{version}/root-aarch64-apple-darwin.tar.gz"
      sha256 ""

      def install
        bin.install "root"
      end
    end

    if Hardware::CPU.intel?
      url "https://github.com/sgr0691/Root/releases/download/v#{version}/root-x86_64-apple-darwin.tar.gz"
      sha256 ""

      def install
        bin.install "root"
      end
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/sgr0691/Root/releases/download/v#{version}/root-x86_64-unknown-linux-gnu.tar.gz"
      sha256 ""

      def install
        bin.install "root"
      end
    end
  end

  test do
    assert_match "root", shell_output("#{bin}/root --version")
  end

  livecheck do
    url :stable
    regex(/^v?(\d+(?:\.\d+)+)$/i)
  end
end
