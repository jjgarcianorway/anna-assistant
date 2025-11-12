class AnnaAssistant < Formula
  desc "Arch Linux AI assistant with ethical AI, consensus, and self-healing"
  homepage "https://github.com/jjgarcianorway/anna-assistant"
  version "1.16.3-alpha.1"
  license "Custom"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/jjgarcianorway/anna-assistant/releases/download/v#{version}/annad-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_INTEL_SHA256"
    elsif Hardware::CPU.arm?
      url "https://github.com/jjgarcianorway/anna-assistant/releases/download/v#{version}/annad-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_ARM_SHA256"
    end
  end

  on_linux do
    url "https://github.com/jjgarcianorway/anna-assistant/releases/download/v#{version}/annad-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_LINUX_SHA256"
  end

  depends_on "openssl@3"

  def install
    bin.install "annad"
    bin.install "annactl"

    # Install default config
    (etc/"anna").mkpath
    (etc/"anna").install "config.toml" if File.exist?("config.toml")

    # Create data directories
    (var/"lib/anna").mkpath
    (var/"lib/anna/keys").mkpath
    (var/"lib/anna/chronos").mkpath
    (var/"lib/anna/collective").mkpath
    (var/"lib/anna/reports").mkpath

    # Install documentation
    doc.install "README.md" if File.exist?("README.md")
    doc.install Dir["docs/*"] if Dir.exist?("docs")
  end

  def post_install
    # Set up proper permissions
    (var/"lib/anna").chmod 0750
    (var/"lib/anna/keys").chmod 0750
    (var/"lib/anna/chronos").chmod 0750
    (var/"lib/anna/collective").chmod 0750
    (var/"lib/anna/reports").chmod 0750
  end

  service do
    run [opt_bin/"annad"]
    keep_alive true
    log_path var/"log/anna/annad.log"
    error_log_path var/"log/anna/annad.err"
    working_dir var/"lib/anna"
  end

  test do
    # Test that binaries are executable and show version
    assert_match version.to_s, shell_output("#{bin}/annactl --version")

    # Test that config directory exists
    assert_predicate etc/"anna", :directory?

    # Test that data directories exist
    assert_predicate var/"lib/anna", :directory?
  end

  def caveats
    <<~EOS
      Anna Assistant has been installed!

      To start the daemon:
        brew services start anna-assistant

      Or run manually:
        annad

      Check status:
        annactl status

      Configuration:
        #{etc}/anna/config.toml

      Data directory:
        #{var}/lib/anna

      Logs:
        #{var}/log/anna/

      For more information:
        https://github.com/jjgarcianorway/anna-assistant
    EOS
  end
end
