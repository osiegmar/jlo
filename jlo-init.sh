#!/usr/bin/env bash
jlo() {
  J="$JLO_HOME/bin/jlo-bin"
  case "$1" in
    env|use)
      # shellcheck disable=SC1090
      . <("$J" "$@")
      ;;
    selfupdate)
      echo -n "Version before update: "
      "$J" version
      /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/java-loader/jlo/refs/heads/main/install.sh)"
      echo -n "Version after update: "
      "$J" version
      ;;
    *)
      "$J" "$@"
      ;;
  esac
}
