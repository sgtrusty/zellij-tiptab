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

_tiptab_token_file="${TMPDIR:-/tmp}/tiptab_token.$$"
_tiptab_pending_pane_id=""
_tiptab_pending_pwd=""
_tiptab_pending_bin=""

_tiptab_flush() {
    if [ -z "$_tiptab_pending_pane_id" ]; then
        return 0
    fi
    if zellij action pipe --name tiptab -- "${ZELLIJ_SESSION_NAME:-}|${_tiptab_pending_pane_id} ${_tiptab_pending_pwd} ${_tiptab_pending_bin:-}" 2>/dev/null; then
        _tiptab_last_pane_id="$_tiptab_pending_pane_id"
        _tiptab_last_pwd="$_tiptab_pending_pwd"
        _tiptab_last_bin="$_tiptab_pending_bin"
    fi
}

_tiptab_report() {
    # Only act inside a Zellij session
    [ -n "${ZELLIJ_PANE_ID:-}" ] || return 0
    local pwd="${PWD:-}"
    [ -n "$pwd" ] || return 0

    _tiptab_pending_pane_id="${ZELLIJ_PANE_ID}"
    _tiptab_pending_pwd="$pwd"
    _tiptab_pending_bin="${1:-}"

    local token
    token=$(cat "$_tiptab_token_file" 2>/dev/null)
    token=$(( ${token:-0} + 1 ))
    printf '%s' "$token" > "$_tiptab_token_file"

    (
        local my_token="$token"
        sleep 0.5
        [ "$(cat "$_tiptab_token_file" 2>/dev/null)" = "$my_token" ] || exit 0
        _tiptab_flush
    ) &
    disown 2>/dev/null || true
}
_tiptab_debug_trap() {
    # DEBUG trap: fires BEFORE each command. Gives us the binary that is about
    # to start / replace the shell (e.g. vim, top, tail -f).
    local bin="${BASH_COMMAND%% *}"
    bin="$(basename "$bin")"

    # Check if it's a valid binary, alias, or function
    local kind
    kind=$(type -t "$bin" 2>/dev/null)
    case "$kind" in
    file | alias | function)
        # Actual binary/alias/function — keep it
        ;;
    builtin | keyword | "")
        # Shell builtin, keyword, or not found — report none
        bin="_tiptab_none"
        ;;
    esac

    # Additional fine-tuning for shell internals that pass the type check
    case "$bin" in
    "[" | printf)
        return
        ;;
    echo | export | alias | unalias | read | type | command | builtin | hash | help | readonly | local | declare | typeset | getopts | true | false | return | exit | shift | ulimit | trap | unset | eval | set | exec | source | . | history | _z | z | cd | _tiptab_*)
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
    export -f _tiptab_report _tiptab_prompt_command _tiptab_debug_trap _tiptab_flush
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
