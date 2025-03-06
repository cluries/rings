#!/bin/bash

commit_update_build() {
    TIMESTAMP=$(date "+%Y%m%d%H%M%S")
    TARGET_FILE="src/lib.rs"
    TMP_FILE=$(mktemp)

    # 处理时间戳
    while IFS= read -r line; do
        if [[ "$line" == pub\ static\ COMMIT_BUILD* ]]; then
            echo "pub static COMMIT_BUILD: &'static str = \"$TIMESTAMP\";"
        else
            echo "$line"
        fi
    done < "$TARGET_FILE" > "$TMP_FILE"
    
    # 新增模块排序处理
    SORT_TMP=$(mktemp)
    grep '^pub mod ' "$TMP_FILE" | sort -f > "$SORT_TMP"
    
    # 重新生成文件（带排序的模块）
    FINAL_TMP=$(mktemp)
    awk -v sort_file="$SORT_TMP" '
        /^pub mod / {
            if (!mods_processed) {
                while ((getline sorted_line < sort_file) > 0)
                    print sorted_line
                close(sort_file)
                mods_processed = 1
            }
            next
        }
        { print }
    ' "$TMP_FILE" > "$FINAL_TMP"

    mv "$FINAL_TMP" "$TARGET_FILE"
    git add "$TARGET_FILE"
    rm "$TMP_FILE" "$SORT_TMP"
}


commit_update_build