# -*-Makefile-*-

test: test-rust test-python test-julia


test-rust colours='':
	just test-rust-pure {{colours}}


test-rust-pure colours='':
	cargo nextest run {{colours}} --workspace --exclude bindings


test-python colours='': python-build-bindings
	pytest -v {{colours}} src bindings


test-python-pure colours='':
	pytest -v {{colours}} src


test-python-bindings colours='': python-build-bindings
	pytest -v {{colours}} bindings


test-julia colours='':
	echo SKIPPING JULIA TESTS FOR NOW
	# julia julia/testme.jl


python-build-bindings profile='default':
	#!/usr/bin/env sh
	cd bindings
	case {{profile}} in
		default )
			cargo build
			ln -fs ../target/debug/libfulano.so fulano.so
			;;
		release )
			cargo build --release
			ln -fs ../target/release/libfulano.so fulano.so
			;;
		* )
			echo Unknown profile {{profile}}
			exit 1
	esac
