build:
	cargo build --release

install-user:
	install -Dm755 target/release/assistantd $(HOME)/.local/bin/assistantd
	install -Dm755 target/release/assistantctl $(HOME)/.local/bin/assistantctl
	install -Dm644 systemd/user/assistantd.service $(HOME)/.config/systemd/user/assistantd.service
	install -Dm644 etc/assistant/policy.d/default.yaml $(HOME)/.config/assistant/policy.d/default.yaml
	install -d $(HOME)/.local/share/assistant/skills
	cp -r var/lib/assistant/skills/* $(HOME)/.local/share/assistant/skills/ || true

test:
	cargo test --all
