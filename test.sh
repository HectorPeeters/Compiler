echo "Building compiler..."
cargo build > /dev/null 2>&1
COMPILE_RESULT=$?
if [ $COMPILE_RESULT -ne 0 ]; then
    echo "Failed building compiler!"
    exit 1
fi

for file in examples/*.sq
do
    echo -n "Running $file..."
    cargo run $file > /dev/null 2>&1
    gcc lib.c output.s
    GCC_RESULT=$?
    if [ $GCC_RESULT -ne 0 ]; then
        echo "Failed running gcc for $file!"
        exit 1
    fi

    OUTPUT=$(./a.out)

    EXPECTED_OUTPUT=$(cat $file.y)

    if [ "$OUTPUT" = "$EXPECTED_OUTPUT" ]; then
        echo " ✓"
    else
        echo " X"
        echo "Expected $EXPECTED_OUTPUT but got $OUTPUT"
        exit 1
    fi
done

rm output.s a.out