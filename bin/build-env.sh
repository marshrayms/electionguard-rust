
# Features of individual crates and used by multiple projects

feats_ul_el_fwnnl=eg-allow-unsafe-code

feats_fwnnl=montgomery
feats_fwnnl=$feats_fwnnl,bits-256
feats_fwnnl=$feats_fwnnl,bits-4096
feats_fwnnl=$feats_fwnnl,num-bigint
feats_fwnnl=$feats_fwnnl,crypto-bigint
#feats_fwnnl=$feats_fwnnl,basic-array
#feats_fwnnl=$feats_fwnnl,hacl

# Features used when acting on test-data-generation or eg
feats_tdgl_el=eg-allow-test-data-generation

# Features used when acting on eg or demo-eg
feats_el_de=eg-allow-insecure-deterministic-csprng
feats_el_de=$feats_el_de,eg-allow-nonstandard-egds-version
#feats_el_de=$feats_el_de,eg-use-toy-params-q7p16
#feats_el_de=$feats_el_de,eg-use-reduced-params-q256p3072
feats_el_de=$feats_el_de,eg-forbid-reduced-params

# Features specific to acting on demo-eg
#feats_de= [none specific, same as eg]

# Features for acting on all targets
feats_all_targets=$feats_ul_el_fwnnl,$feats_fwnnl,$feats_tdgl_el,$feats_el_de

# Flags for referencing specfic crates and combining multiple sets of features

ndf_f='--no-default-features --features'

# Flags when acting on only lib `util`
flags_ul="--lib -p util $ndf_f $feats_ul_el_fwnnl"

# Flags when acting on only lib `fixed-width-nonnegative`
flags_fwnnl="--lib -p fixed-width-nonnegative $ndf_f $feats_ul_el_fwnnl,$feats_fwnnl"

flags_tdgl="--lib -p test-data-generation $ndf_f $feats_tdgl_el"

flags_el="--lib -p eg $ndf_f $feats_ul_el_fwnnl,$feats_tdgl_el,$feats_el_de"

flags_de="--bin demo-eg $ndf_f $feats_ul_el_fwnnl,$feats_tdgl_el,$feats_el_de"

flags_all_targets="$ndf_f $feats_all_targets"

unset -v ndf_f feats_ul_el_fwnnl feats_fwnnl feats_tdgl_el feats_el_de feats_all_targets

cargo_build_flags='--all-targets'
cargo_clippy_flags='--workspace'
cargo_watch_flags='--why --no-restart --watch-when-idle --ignore *.pending-snap'

cargo_test_flags='--locked --offline --no-fail-fast'
cargo_test_args='--test-threads=1'

cargo_insta_flags_all_targets='--all'
cargo_insta_test_flags=''

printf '# Cargo aliases:\n'
printf '#\n'
printf '#   clippy:   %s ...\n' "$cargo_clippy_flags"
printf '#   building: %s ...\n' "$cargo_build_flags"
printf '#   watch:    %s ...\n' "$cargo_watch_flags"

cargo_profile_flags='--profile=test'

printf '#\n'
printf '# For %s:\n' "$cargo_profile_flags"
printf '#     just     ul: %s\n' "$flags_ul"
printf '#     just   tdgl: %s\n' "$flags_tdgl"
printf '#     just     el: %s\n' "$flags_el"
printf '#     just  fwnnl: %s\n' "$flags_fwnnl"
printf '#     just     de: %s\n' "$flags_de"
printf '#     all targets: %s\n' "$flags_all_targets"
printf '#\n'

# -p util --lib
export CARGO_ALIAS_BUL="build $cargo_profile_flags $cargo_build_flags $flags_ul"
printf 'cargo bul:    cargo build ... %s\n' "$flags_ul"
export CARGO_ALIAS_WBUL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_BUL"
printf 'cargo wbul:   cargo watch build ... %s\n' "$flags_ul"
export CARGO_ALIAS_TUL="test $cargo_profile_flags $cargo_test_flags $flags_ul -- $cargo_test_args"
printf 'cargo tul:    cargo test  ... %s\n' "$flags_ul"
export CARGO_ALIAS_TULI="test $cargo_profile_flags $cargo_test_flags $flags_ul -- --ignored $cargo_test_args"
printf 'cargo tuli:   cargo test ... %s -- --ignored ...\n' "$flags_ul"
export CARGO_ALIAS_WTUL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_TUL"
printf 'cargo wtul:   cargo watch test  ... %s\n' "$flags_ul"

# -p test-data-generation --lib
export CARGO_ALIAS_BTDGL="build $cargo_profile_flags $cargo_build_flags $flags_tdgl"
printf 'cargo btdgl:  cargo build ... %s\n' "$flags_tdgl"
export CARGO_ALIAS_WBTDGL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_BTDGL"
printf 'cargo wbtdgl: cargo watch build ... %s\n' "$flags_tdgl"
export CARGO_ALIAS_TTDGL="test $cargo_profile_flags $cargo_test_flags $flags_tdgl -- $cargo_test_args"
printf 'cargo ttdgl:  cargo test  ... %s -- ...\n' "$flags_tdgl"
export CARGO_ALIAS_WTTDGL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_TTDGL"
printf 'cargo wttdgl: cargo watch test  ... %s\n' "$flags_tdgl"

# -p eg --lib
export CARGO_ALIAS_BEL="build $cargo_profile_flags $cargo_build_flags $flags_el"
printf 'cargo bel:   cargo build ... %s\n' "$flags_el"
export CARGO_ALIAS_WBEL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_BEL"
printf 'cargo wbel:  cargo watch build ... %s\n' "$flags_el"
export CARGO_ALIAS_TEL="test $cargo_profile_flags $cargo_test_flags $flags_el -- $cargo_test_args"
printf 'cargo tel:   cargo test  ... %s -- ...\n' "$flags_el"
export CARGO_ALIAS_TELI="test $cargo_profile_flags $cargo_test_flags $flags_el -- --ignored $cargo_test_args"
printf 'cargo teli:  cargo test ... %s -- --ignored ...\n' "$flags_el"
export CARGO_ALIAS_WTEL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_TEL"
printf 'cargo wtel:  cargo watch test  ... %s\n' "$flags_el"

# -p fixed-width-nonnegative --lib
export CARGO_ALIAS_BFWNNL="build $cargo_profile_flags $cargo_build_flags $flags_fwnnl"
printf 'cargo bel:   cargo build ... %s\n' "$flags_fwnnl"
export CARGO_ALIAS_WBFWNNL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_BFWNNL"
printf 'cargo wbel:  cargo watch build ... %s\n' "$flags_fwnnl"
export CARGO_ALIAS_TFWNNL="test $cargo_profile_flags $cargo_test_flags $flags_fwnnl -- $cargo_test_args"
printf 'cargo tel:   cargo test  ... %s -- ...\n' "$flags_fwnnl"
export CARGO_ALIAS_WTFWNNL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_TFWNNL"
printf 'cargo wtel:  cargo watch test  ... %s\n' "$flags_fwnnl"

export CARGO_ALIAS_RDE="run $cargo_profile_flags $flags_de"
printf 'cargo rde:   cargo run -bin demo-eg ...\n'
export CARGO_ALIAS_WRDE="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_RDE"
printf 'cargo wrde:  cargo watch -- cargo run --bin demo-eg ...\n'

export CARGO_ALIAS_B="build $cargo_profile_flags $cargo_build_flags $flags_all_targets"
printf 'cargo b:     cargo build ...\n'
export CARGO_ALIAS_WB="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_B"
printf 'cargo wb:    cargo watch build ...\n'

export CARGO_ALIAS_C="clippy $cargo_profile_flags $cargo_clippy_flags $cargo_build_flags $flags_all_targets"
printf 'cargo c:     cargo clippy ...\n'
export CARGO_ALIAS_T="test $cargo_profile_flags $cargo_test_flags $flags_all_targets -- $cargo_test_args"
printf 'cargo t:     cargo test ... -- ...\n'
export CARGO_ALIAS_WT="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_T"
printf 'cargo wt:    cargo watch test  ...\n'
export CARGO_ALIAS_TI="test $cargo_profile_flags $cargo_test_flags $flags_all_targets -- --ignored $cargo_test_args"
printf 'cargo ti:    cargo test ... -- --ignored ...)\n'

export CARGO_ALIAS_ITRUL="insta test --review $cargo_profile_flags $cargo_insta_test_flags $flags_ul -- $cargo_test_args"
printf 'cargo itrul:   cargo insta test --review ... %s\n' "$flags_ul"
export CARGO_ALIAS_WITRUL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_ITRUL"
printf 'cargo witrul: cargo watch -- cargo insta test --review ... %s\n' "$flags_ul"

export CARGO_ALIAS_ITRTDGL="insta test --review $cargo_profile_flags $cargo_insta_test_flags $flags_tdgl -- $cargo_test_args"
printf 'cargo itrtdgl: cargo insta test --review ... %s\n' "$flags_tdgl"
export CARGO_ALIAS_WITRTDGL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_ITRTDGL"
printf 'cargo witrtdgl: cargo watch -- cargo insta test --review ... %s\n' "$flags_tdgl"

export CARGO_ALIAS_ITREL="insta test --review $cargo_profile_flags $cargo_insta_test_flags $flags_el -- $cargo_test_args"
printf 'cargo itrel:   cargo insta test --review ... %s\n' "$flags_el"
export CARGO_ALIAS_WITREL="watch $cargo_watch_flags -- cargo $CARGO_ALIAS_ITREL"
printf 'cargo witrel: cargo watch -- cargo insta test --review ... %s\n' "$flags_el"

export CARGO_ALIAS_ITRELI="insta test --review $cargo_profile_flags $cargo_insta_test_flags $flags_el -- --ignored $cargo_test_args"
printf 'cargo itreli:  cargo insta test --review ... %s -- --ignored ...\n' "$flags_el"
export CARGO_ALIAS_ITR="insta test --review $cargo_profile_flags $cargo_insta_flags_all_targets $cargo_insta_test_flags $flags_all_targets -- $cargo_test_args"
printf 'cargo itr:     cargo insta test --review ...\n'
export CARGO_ALIAS_ITRI="insta test --review $cargo_profile_flags $cargo_insta_flags_all_targets $cargo_insta_test_flags $flags_all_targets -- --ignored  $cargo_test_args"
printf 'cargo itri:    cargo insta test --review ... -- --ignored ...\n'
export CARGO_ALIAS_IR="insta review $cargo_insta_flags_all_targets"
printf 'cargo ir:      cargo insta review ...\n'

unset -v cargo_profile_flags
unset -v cargo_clippy_flags cargo_watch_flags
unset -v cargo_insta_flags_all_targets cargo_insta_test_flags
unset -v flags_el flags_de flags_ul flags_tdgl flags_all_targets
unset -v cargo_build_flags
unset -v cargo_test_flags cargo_test_args

echo

export CARGO_ALIAS_FA="fmt --all"
printf 'cargo %s:   cargo fmt --all\n' 'fa'

echo

export CARGO_INCREMENTAL=0

if [ -z "${CARGO_INCREMENTAL:-}" ]; then
    printf 'CARGO_INCREMENTAL is null or unset\n'
else
    printf 'CARGO_INCREMENTAL=%s\n' "$CARGO_INCREMENTAL" | cat -t
fi
printf '\n'

toplevel_dir=$(git rev-parse --show-toplevel)
toplevel_dir_exit_status=$?
if [ $toplevel_dir_exit_status != 0 ]; then
    cat <<EOD >&2
Couldn't figure toplevel dir.
    git rev-parse --show-toplevel
Exited with status: $toplevel_dir_exit_status
EOD
    unset -v toplevel_dir toplevel_dir_exit_status
    return 123
else
    set -x
    alias egbuildenv='cd '"$toplevel_dir"'/src && . ../bin/build-env.sh'
    alias egclsbuildenv='tput reset; egbuildenv'
    alias egfa='    egclsbuildenv && cargo fa'
    :
    alias egbul='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_BUL"; cargo bul'
    alias egwbul='  egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WBUL"; cargo wbul'
    alias egtul='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_TUL"; cargo tul'
    alias egwtul='  egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WTUL"; cargo wtul'
    alias egtuli='  egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_TULI"; cargo tuli'
    :
    alias egbfwnnl=' egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_BFWNNL"; cargo bfwnnl'
    alias egwbfwnnl='egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WBFWNNL"; cargo bfwnnl'
    alias egtfwnnl=' egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_TFWNNL"; cargo tfwnnl'
    alias egwtfwnnl='egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WTFWNNL"; cargo wtfwnnl'
    :
    alias egbtdgl=' egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_BTDGL"; cargo btdgl'
    alias egwbtdgl='egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WBTDGL"; cargo btdgl'
    alias egttdgl=' egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_TTDGL"; cargo ttdgl'
    alias egwttdgl='egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WTTDGL"; cargo wttdgl'
    :
    alias egbel='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_BEL"; cargo bel'
    alias egwbel='  egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WBEL"; cargo wbel'
    alias egtel='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_TEL"; cargo tel'
    alias egwtel='  egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WTEL"; cargo wtel'
    alias egteli='  egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_TELI"; cargo teli'
    :
    alias egrde='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_RDE"; cargo rde'
    alias egwrde='  egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WRDE"; cargo wrde'
    :
    alias egb='     egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_B"; cargo b'
    alias egwb='    egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WB"; cargo wb'
    :
    alias egc='     egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_C"; cargo c'
    :
    alias egt='     egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_T"; cargo t'
    alias egwt='    egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WT"; cargo wt'
    alias egti='    egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_TI"; cargo ti'
    alias egwti='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WTI"; cargo wti'
    :
    alias egitrul='    egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_ITRUL"; cargo itrul'
    alias egwitrul='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WITRUL"; cargo witrul'
    alias egitrfwnnl=' egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_ITRFWNNL"; cargo itrfwnnl'
    alias egwitrfwnnl='egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WITRFWNNL"; cargo witrfwnnl'
    alias egitrtdgl='  egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_ITRTDGL"; cargo itrtdgl'
    alias egwitrtdgl=' egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WITRTDGL"; cargo witrtdgl'
    alias egitrel='    egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_ITREL"; cargo itrel'
    alias egwitrel='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_WITREL"; cargo witrel'
    alias egitreli='   egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_ITRELI"; cargo itreli'
    alias egitr='      egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_ITR"; cargo itr'
    alias egitri='     egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_ITRI"; cargo itri'
    alias egir='       egclsbuildenv && printf "cargo %s\n\n" "$CARGO_ALIAS_IR"; cargo ir'
    :
    set +x
    echo

    unset -v toplevel_dir toplevel_dir_exit_status
    return 0
fi
