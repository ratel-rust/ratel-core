echo ""
echo ""
echo "CHECKING: clippy"
echo "#######################"
echo ""

cargo +$CLIPPY_TOOLCHAIN clippy || exit

echo ""
echo ""
echo "TESTING: ratel + crates"
echo "#######################"
echo ""

cargo test || exit

echo ""
echo ""
echo "TESTING: ffi"
echo "############"
echo ""

cd ffi
npm i || exit
npm test || exit
cd ..
