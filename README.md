# vsub
A visual regex substitution tool

## Commands
### `w`
**W**rite (save) the edited buffer to the opened file

### `s/<fpat>/<spat>/`
**S**ubstitutes all occurrences of `fpat` with `spat`. `$c` = access capture group `c`, `$0` = whole match

### `s/<pat>/`
Removes all occurrences of `pat` (**s**ubstitutes with nothing)

### `p/<pat>/`
Highlights all occurrences of `pat` (**p**reviews what `s` would remove)
