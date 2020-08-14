echo "Building compiler..."
cargo build > /dev/null 2>&1
COMPILE_RESULT=$?
if [ $COMPILE_RESULT -ne 0 ]; then
    echo "Failed building compiler!"
    exit 1
fi

echo "\nRunning tests..."
for file in examples/*.sq
do
    echo -n "Running $file..."
    cargo run $file > /dev/null 2>&1
    CARGO_RESULT=$?
    if [ $CARGO_RESULT -ne 0 ]; then
        echo
        echo "Failed running cargo for $file!"
        exit 1
    fi

    gcc lib.c output.s
    GCC_RESULT=$?
    if [ $GCC_RESULT -ne 0 ]; then
        echo
        echo "Failed running gcc for $file!"
        exit 1
    fi

    OUTPUT=$(./a.out)

    EXPECTED_OUTPUT=$(cat $file.y)

    bold=$(tput bold)
    normal=$(tput sgr0)

    if [ "$OUTPUT" = "$EXPECTED_OUTPUT" ]; then
        echo " ${bold}✓${normal}"
    else
        echo " ${bold}⨯${normal}"
        echo -e "\n${bold}Expected:${normal}"
        echo -e "$EXPECTED_OUTPUT"
        echo -e "\n${bold}But got:${normal}"
        echo -e "$OUTPUT"
        exit 1
    fi
done

echo "\nRunnig failing tests..."

for file in examples/failing/*.sq
do
    echo -n "Runnig $file..."
    cargo run $file > /dev/null 2>&1
    CARGO_RESULT=$?
    if [ $CARGO_RESULT -ne 0 ]; then
        echo " ${bold}✓${normal}"
    else
        echo " ${bold}⨯${normal}"
        exit 1 
    fi
done

rm output.s a.out
