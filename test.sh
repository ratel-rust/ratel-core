set -e

echo ""
echo ""
echo "CHECKING: clippy"
echo "#######################"
echo ""

if [ $TRAVIS_RUST_VERSION = $CLIPPY_TOOLCHAIN ]; then
  cargo +$CLIPPY_TOOLCHAIN clippy
fi

echo ""
echo ""
echo "TESTING: ratel + crates"
echo "#######################"
echo ""

cargo test

echo ""
echo ""
echo "TESTING: ffi"
echo "############"
echo ""

cd ffi
npm i
npm test
cd ..
