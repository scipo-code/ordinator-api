/id\.resources/ {
  first = $0
  if (getline second) {
    if (second ~ /id\.id/) {
      print second
      print first
      next
      
    } else {
      print first
      print second
      next
    }
  }
}
{ print }
