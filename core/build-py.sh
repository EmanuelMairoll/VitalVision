### BUILDING FOR PYTHON ON MACOS ARM ###

NAME="vvcore"
HEADERPATH="bindings/${NAME}FFI.h"
BINDINGPATH="bindings/${NAME}.py"
TARGETDIR="target"
RELDIR="release"
STATIC_LIB_NAME="lib${NAME}.a"
OUTDIR="../signal/vvcore"

# only aarch64 for now
cargo build --target aarch64-apple-darwin --release

rm -rf "${OUTDIR}/"
mkdir -p "${OUTDIR}"
cp "${TARGETDIR}/aarch64-apple-darwin/${RELDIR}/${STATIC_LIB_NAME}" "${OUTDIR}/libuniffi_vvcore.dylib"
cp "${BINDINGPATH}" "${OUTDIR}/"
