# This script takes care of building your crate and packaging it for release

set -ex

generate_docs() {
    cargo doc --all --verbose
    echo '<head><meta http-equiv="refresh" content="0; URL=gcode/index.html" /></head>' > target/doc/index.html
}

generate_bundle() {
    test -f Cargo.lock || cargo generate-lockfile

    cross build --target $TARGET --release
    cp target/$TARGET/release/libgcode.a $stage/
    if [ $TRAVIS_OS_NAME = linux ]; then
        cp target/$TARGET/release/libgcode.so $stage/
    else
        cp target/$TARGET/release/libgcode.dylib $stage/
    fi

    # We need the cbindgen tool for generating our header file
    if [ -f $HOME/.cargo/env ]; then source $HOME/.cargo/env; fi
    command -v cbindgen >/dev/null 2>&1 || cargo install --debug cbindgen

    cbindgen --output $stage/gcode.h

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    generate_docs
    generate_bundle
}

main
