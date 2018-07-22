# This script takes care of testing your crate

set -ex

main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release

    # Make sure our README example stays up-to-date
    cross rustdoc README.md --test --extern gcode=target/debug/deps/libgcode.rlib -L target/debug/deps

    # We also want to test the C example
    if [ $TRAVIS_OS_NAME = linux ]; then
        cd ffi-example
        make && LD_LIBRARY_PATH=. ./example
    fi
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
