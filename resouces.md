how should we handle resources?

have src="@{res_folder:resource_name}"
or url("@{res_folder:resource_name}")

we then replace these in post.

for dev:
	@{icons:foobar} => ../res/icons/foobar-9.png

for ao3:
	@{icons:foobar} => taxen99.github.io/wao3/{some small name}.png

