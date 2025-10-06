#!/usr/bin/env sh

set -eu

JLO_HOME="$HOME/.jlo"
JLO_BIN_DIR="$JLO_HOME/bin"
JLO_BIN="$JLO_BIN_DIR/jlo-bin"
JLO_INIT="$JLO_BIN_DIR/jlo-init.sh"
JLO_URL="https://github.com/osiegmar/jlo/releases/download/test/jlo-bin"

mkdir -p "$JLO_BIN_DIR"

if ! curl -fsSL https://github.com/osiegmar/jlo/releases/download/test/jlo-bin -o "$JLO_BIN"; then
  echo "Failed to download jlo binary from $JLO_URL" >&2
  exit 1
fi
chmod 755 "$JLO_BIN"

cat <<'EOF' > "$JLO_INIT"
#!/usr/bin/env zsh

jlo() {
  J="$HOME/.jlo/bin/jlo-bin"
  case "$1" in
    env|use)
      . <("$J" "$@")
      ;;
    *)
      "$J" "$@"
      ;;
  esac
}

jlo_after_cd() {
  [[ -f ".jlorc" ]] && jlo env
}

autoload -U add-zsh-hook
add-zsh-hook chpwd jlo_after_cd
EOF

chmod 755 "$JLO_INIT"

cat <<EOF
Successfully installed J'lo to $JLO_HOME.

To use it, add the following to your shell configuration file (e.g. ~/.zshrc or ~/.bashrc):

export JLO_HOME="\$HOME/.jlo"
[[ -s "\$JLO_HOME/bin/jlo-init.sh" ]] && source "\$JLO_HOME/bin/jlo-init.sh"

Then restart your terminal or execute the above lines in your current shell session.

After that, you can use the 'jlo' command to manage your Java environments.
EOF
