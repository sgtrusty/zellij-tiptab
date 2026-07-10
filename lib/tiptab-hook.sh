#!/usr/bin/env bash
# tiptab-hook.sh — feed the tiptab plugin the working directory + foreground
# binary of each pane.
#
# The tiptab plugin derives binary/git/folder tab names from a pane's cwd and
# foreground binary. Zellij's PaneInfo does not expose either, so this hook
# emits a `tiptab` pipe carrying "<pane_id> <pwd> <bin>" on every prompt AND
# on every command launch (via the bash DEBUG trap).
#
# Source it from your shell rc, e.g.:
#   source /path/to/tiptab-hook.sh

_tiptab_last_ts=0

_tiptab_report() {
    # Only act inside a Zellij session
    [ -n "${ZELLIJ_PANE_ID:-}" ] || return 0
    local pwd="${PWD:-}"
    [ -n "$pwd" ] || return 0

    # Calculate current time in milliseconds (or use floats with EPOCHREALTIME)
    local now=$(date +%s%3N)
    local delta=$(( now - _tiptab_last_ts ))

    # Debounce Logic:
    # 1. Skip if data hasn't changed (original)
    # 2. Skip if we sent an update less than 500ms ago
    if [ "${_tiptab_last_pane_id:-}" = "${ZELLIJ_PANE_ID}" ] \
       && [ "${_tiptab_last_pwd:-}" = "$pwd" ] \
       && [ "${_tiptab_last_bin:-}" = "${1:-}" ] \
       || [ $delta -lt 500 ]; then
        return 0
    fi

    local tab_pos=$(zellij action current-tab-info 2>/dev/null | sed -n 's/^position: //p')

    # Payload
    _tiptab_last_ts=$now
    if zellij action pipe --name tiptab -- "${tab_pos} ${pwd} ${1:-}" 2>/dev/null; then
        _tiptab_last_pane_id="${ZELLIJ_PANE_ID}"
        _tiptab_last_pwd="$pwd"
        _tiptab_last_bin="${1:-}"
    fi
}
_tiptab_debug_trap() {
    # DEBUG trap: fires BEFORE each command. Gives us the binary that is about
    # to start / replace the shell (e.g. vim, top, tail -f).
    local bin="${BASH_COMMAND%% *}"
    bin="$(basename "$bin")"
    case "$bin" in
        "["|printf)
            return;
            ;;
        source|.|history|eval|exec|set|shift|ulimit|trap|unset|true|false|return|exit|_z|z|cd|echo|export|alias|unalias|read|type|command|builtin|hash|help|readonly|local|declare|typeset|getopts|_tiptab_*)
            bin="_tiptab_none"
            ;;
    esac
    _tiptab_report "$bin"
}
_tiptab_prompt_command() {
    # Fires after each command exits (or on plain prompt). Sends empty bin
    # so the plugin reverts to PWD-based naming immediately.
    _tiptab_report "_tiptab_none"
}
# --- bash ---
if [ -n "${BASH_VERSION:-}" ]; then
    # Chain onto PROMPT_COMMAND instead of clobbering it, so tools sourced
    # either before or after this hook keep working.
    # case ";${PROMPT_COMMAND:-};" in
    # *_tiptab_prompt_command*) ;;
    # *) export PROMPT_COMMAND="_tiptab_prompt_command;${PROMPT_COMMAND:-}" ;;
    # esac
    export -f _tiptab_report _tiptab_prompt_command _tiptab_debug_trap 
    # Chain onto any pre-existing DEBUG trap instead of overwriting it —
    # sourcing this before or after tools like direnv/atuin/starship (which
    # also use the DEBUG trap) would otherwise silently break one or the
    # other, depending on source order.
    _tiptab_existing_debug_trap="$(trap -p DEBUG | sed -e "s/^trap -- '//" -e "s/' DEBUG$//")"
    if [ -n "$_tiptab_existing_debug_trap" ] && [[ "$_tiptab_existing_debug_trap" != *_tiptab_debug_trap* ]]; then
        # shellcheck disable=SC2064
        trap "_tiptab_debug_trap; ${_tiptab_existing_debug_trap}" DEBUG
    else
        trap '_tiptab_debug_trap' DEBUG
    fi
    unset _tiptab_existing_debug_trap
fi
# --- zsh ---
if [ -n "${ZSH_VERSION:-}" ]; then
    if [[ -o interactive ]]; then
        if typeset -f add-zsh-hook >/dev/null 2>&1; then
            add-zsh-hook preexec _tiptab_prompt_command
            add-zsh-hook precmd _tiptab_report
        else
            typeset -ag preexec_functions
            case ";${preexec_functions[*]};" in
            *_tiptab_prompt_command*) ;;
            *) preexec_functions+=(_tiptab_prompt_command) ;;
            esac
            typeset -ag precmd_functions
            case ";${precmd_functions[*]};" in
            *_tiptab_report*) ;;
            *) precmd_functions+=(_tiptab_report) ;;
            esac
        fi
    fi
fi
