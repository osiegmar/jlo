#!/usr/bin/env sh

jlo_after_cd() {
  [ -f ".jlorc" ] && jlo env
}

if [ -n "$ZSH_VERSION" ]; then
  autoload -U add-zsh-hook
  add-zsh-hook chpwd jlo_after_cd
  jlo_after_cd
elif [ -n "$BASH_VERSION" ]; then
  PROMPT_COMMAND="jlo_after_cd; $PROMPT_COMMAND"
  jlo_after_cd
fi

# Immediate call for fresh spawned shells
jlo_after_cd
