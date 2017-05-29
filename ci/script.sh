# This script takes care of testing your crate

set -ex

main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    # When we're compiling on nightly we need to add the "nightly" feature
    if [ ! -z $NIGHTLY ]; then
      cross test --target $TARGET --features=nightly
      cross test --target $TARGET --release --features=nightly
    else
      cross test --target $TARGET 
      cross test --target $TARGET --release
    fi

}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
