#!/usr/bin/env bash

SHELL_LIMIT=4

build_shell_widget() {
  local ignored="build_shell_string_fake() { :; }"
  echo "$SHELL_LIMIT"
}

function render_shell_widget() {
  build_shell_widget
}
