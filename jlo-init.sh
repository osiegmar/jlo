#!/usr/bin/env bash
jlo() {
  J="$JLO_HOME/bin/jlo-bin"
  case "$1" in
    env|use)
      # shellcheck disable=SC1090
      . <("$J" "$@")
      ;;
    selfupdate)
      /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/osiegmar/jlo/refs/heads/main/install.sh)"
      ;;
    *)
      "$J" "$@"
      ;;
  esac
}
