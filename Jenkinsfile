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
                sh "cargo test"
                sh "cargo test --release"
            }
        }
    }
}
