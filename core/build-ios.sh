### BUILDING FOR IOS/MACOS ###

NAME="vvcore"
HEADERPATH="bindings/${NAME}FFI.h"
BINDINGPATH="bindings/${NAME}.swift"
TARGETDIR="target"
RELDIR="release"
STATIC_LIB_NAME="lib${NAME}.a"
OUTDIR="../ios/Frameworks"
NEW_HEADER_DIR="bindings/include"

# only aarch64 for now
cargo build --target aarch64-apple-darwin --release
cargo build --target aarch64-apple-ios --release
cargo build --target aarch64-apple-ios-sim --release

mkdir -p "${NEW_HEADER_DIR}"
cp "${HEADERPATH}" "${NEW_HEADER_DIR}/"
cp "bindings/${NAME}FFI.modulemap" "${NEW_HEADER_DIR}/module.modulemap"

rm -rf "${OUTDIR}/${NAME}_framework.xcframework"

xcodebuild -create-xcframework \
    -library "${TARGETDIR}/aarch64-apple-darwin/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/aarch64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/aarch64-apple-ios-sim/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -output "${OUTDIR}/${NAME}_framework.xcframework"

cp -f "${BINDINGPATH}" "${OUTDIR}/"
