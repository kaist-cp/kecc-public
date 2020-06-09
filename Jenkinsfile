def setupRust() {
    sh "rustup component add rustfmt clippy"
    sh "rustup install nightly"
    sh "cargo update"
    sh "cargo"
}

pipeline {
    agent {
        docker {
            image 'rust:latest'
        }
    }

    stages {
        stage('Rustfmt') {
            steps {
                setupRust()
                sh "cargo fmt --all -- --check"
            }
        }
        stage('Clippy') {
            steps {
                setupRust()
                sh "cargo clippy --all"
            }
        }
        stage('Build') {
            steps {
                setupRust()
                sh "cargo build"
                sh "cargo build --release"
            }
        }
        stage('Test') {
            steps {
                setupRust()
                // When `cargo test` runs, the function `it_works()` is called in a new thread.
                // The stack size of a new thread is `2 MiB` on Linux, and this small stack size 
                // can cause `stack-overflow` error when testing stack-intensive code.
                // For this reason, we need to increase the default size of stack to `8 MiB`.
                // TODO: delete `--skip test_examples_asmgen`
                sh "RUST_MIN_STACK=8388608 cargo test -- --skip test_examples_asmgen --skip test_examples_end_to_end"
                sh "RUST_MIN_STACK=8388608 cargo test --release -- --skip test_examples_asmgen --skip test_examples_end_to_end"
            }
        }
    }
}
