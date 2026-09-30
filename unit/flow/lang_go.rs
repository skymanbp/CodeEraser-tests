//! Go lowered as the table says (plan v2.31 step 4 A2): each block one
//! rule of design booklet §5.1 on the Go grammar, its rows the first
//! unit's statements, variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
go @@ loops @@ rules 4, 5, 8: a for clause is init · loop · update label; a range with `:=` declares; `for {}` is infinite; `select {}` never returns
func f(xs []int) int {
	s := 0
	for i := 0; i < len(xs); i++ {
		if xs[i] < 0 {
			continue
		}
		s += xs[i]
	}
	for _, x := range xs {
		s -= x
	}
	for {
		select {}
	}
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 3 0 0
s 3 2 0 0 0
s 4 3 2 0 0
s 5 4 0 0 0
s 6 5 13 0 8
s 7 3 1 0 0
s 8 2 14 0 0
s 9 8 1 0 0
s 10 -1 3 0 0
s 11 10 0 0 0
s 12 11 1 0 0
s 13 -1 3 2 0
s 14 13 0 0 0
s 15 14 15 0 0
v 0 -1 1 xs
v 1 0 0 s
v 2 1 0 i
v 3 10 0 x
x 0 1 1
x 1 2 1
x 2 2 0
x 2 0 0
x 4 0 0
x 4 2 0
x 7 0 0
x 7 2 0
x 7 1 2
x 9 2 2
x 10 0 0
x 10 3 1
x 12 3 0
x 12 1 2
====
go @@ switch @@ rules 6, 8 and ruling 6: a trailing fallthrough sets its case's flag; the init is scoped to the switch; a type switch binds; a bare return reads the named results
func f(k int, v any) (n int) {
	switch m := k * 2; m {
	case 1:
		n = 1
		fallthrough
	case 2:
		n = 2
	default:
		n = 3
	}
	switch t := v.(type) {
	case string:
		n += len(t)
	}
	return
}
----
s 0 -1 4 1 0
s 1 0 5 4 0
s 2 1 1 0 0
s 3 0 5 0 0
s 4 3 1 0 0
s 5 0 5 0 0
s 6 5 1 0 0
s 7 -1 4 0 0
s 8 7 5 0 0
s 9 8 1 0 0
s 10 -1 9 0 0
v 0 -1 1 k
v 1 -1 1 v
v 2 -1 1 n
v 3 0 0 m
v 4 7 0 t
x 0 0 0
x 0 3 1
x 0 3 0
x 2 2 1
x 4 2 1
x 6 2 1
x 7 1 0
x 7 4 1
x 9 4 0
x 9 2 2
x 10 2 0
====
go @@ redeclare @@ rules 8, 10: `:=` writes a name its scope holds; an if's init is scoped to the if; `_` is no variable; `&p` takes the address
func f() error {
	a, err := g()
	b, err := h(a)
	if v, err := k(b); err != nil {
		return err
	} else {
		use(v)
	}
	_ = b
	p := 0
	q(&p)
	return err
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 2 1 0
s 3 2 0 0 0
s 4 3 9 0 0
s 5 2 0 0 0
s 6 5 1 0 0
s 7 -1 1 0 0
s 8 -1 1 0 0
s 9 -1 1 0 0
s 10 -1 9 0 0
v 0 0 0 a
v 1 0 0 err
v 2 1 0 b
v 3 2 0 v
v 4 2 0 err
v 5 8 8 p
x 0 0 1
x 0 1 1
x 1 0 0
x 1 2 1
x 1 1 1
x 2 2 0
x 2 3 1
x 2 4 1
x 2 4 0
x 4 4 0
x 6 3 0
x 7 2 0
x 8 5 1
x 9 5 0
x 10 1 0
====
go @@ select-goto @@ rules 3, 6: a select always has its default, a receive arm declares; a goto reaches its label; os.Exit never returns
func f(ch chan int, done chan bool) int {
	n := 0
loop:
	select {
	case v := <-ch:
		n += v
		goto loop
	case <-done:
		return n
	}
	os.Exit(1)
	return 0
}
----
s 0 -1 1 0 0
s 1 -1 14 0 0
s 2 1 4 1 0
s 3 2 5 0 0
s 4 3 1 0 0
s 5 3 13 0 1
s 6 2 5 0 0
s 7 6 9 0 0
s 8 -1 15 0 0
s 9 -1 9 0 0
v 0 -1 1 ch
v 1 -1 1 done
v 2 0 0 n
v 3 3 0 v
x 0 2 1
x 3 0 0
x 3 3 1
x 4 3 0
x 4 2 2
x 6 1 0
x 7 2 0
====
go @@ closure @@ rules 3, 10 and ruling 6: a func literal captures the named results it names; panic never returns; `return a, b` reads no named result
func f(x int) (res int, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("%v", r)
		}
	}()
	if x < 0 {
		panic("neg")
	}
	res = x
	return res, nil
}
----
s 0 -1 1 0 0
s 1 -1 2 0 0
s 2 1 0 0 0
s 3 2 15 0 0
s 4 -1 1 0 0
s 5 -1 9 0 0
v 0 -1 1 x
v 1 -1 1 res
v 2 -1 3 err
x 1 0 0
x 4 0 0
x 4 1 1
x 5 1 0
====
go @@ var-labels @@ rules 4, 8: var declarations declare each name; a labelled break leaves the outer loop
func f(grid [][]int) int {
	var total, count int
	var (
		limit = 10
	)
outer:
	for _, row := range grid {
		for _, c := range row {
			if c > limit {
				break outer
			}
			total += c
			count++
		}
	}
	return total / count
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 14 0 0
s 3 2 3 0 0
s 4 3 0 0 0
s 5 4 3 0 0
s 6 5 0 0 0
s 7 6 2 0 0
s 8 7 0 0 0
s 9 8 11 0 3
s 10 6 1 0 0
s 11 6 1 0 0
s 12 -1 9 0 0
v 0 -1 1 grid
v 1 0 0 total
v 2 0 0 count
v 3 1 0 limit
v 4 3 0 row
v 5 5 0 c
x 1 3 1
x 3 0 0
x 3 4 1
x 5 4 0
x 5 5 1
x 7 5 0
x 7 3 0
x 10 5 0
x 10 1 2
x 11 2 2
x 12 1 0
x 12 2 0
"#;

#[test]
fn go_lowers_as_the_table_says() {
    run_table(TABLE);
}
