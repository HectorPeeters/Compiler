for file in examples/*.sq
do
    echo -n "Running $file..."
    cargo run $file > /dev/null 2>&1
    gcc output.s

    OUTPUT=$(./a.out)

    EXPECTED_OUTPUT=$(cat $file.y)

    if [ "$OUTPUT" = "$EXPECTED_OUTPUT" ]; then
        echo " ✓"
    else
        echo ""
    fi
done