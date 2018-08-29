echo ""
echo ""
echo "TESTING: ratel + crates"
echo "#######################"
echo ""

cargo +nightly clippy || exit
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
