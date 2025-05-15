/id\.[^=]+=/ {


# split($0, tmp, "=")
# gsub(/^[ \t]+|[ \t]+$/, "", tmp[2])
vals[++cnt] = $3


if (cnt == 3) {
  printf "id = [ %s, %s, %s ]\n", vals[1], vals[2], vals[3]
  delete vals
  cnt = 0
}
next
}

{ print }


# /^[ \t]*id\.[^=]+=[ \t]*[^ \t][^#]*/ {

#     # Grab everything after the first "=" and trim leading/trailing blanks.
#     split($0, tmp, "=")
#     gsub(/^[ \t]+|[ \t]+$/, "", tmp[2])
#     vals[++cnt] = tmp[2]

#     # We’ve collected three – emit and reset
#     if (cnt == 3) {
#         printf "id = [ %s, %s, %s ]\n", vals[1], vals[2], vals[3]
#         delete vals
#         cnt = 0
#     }

#     next        # don’t print the individual id.* line
# }
