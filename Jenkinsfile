pipeline {
    agent { 
        docker {
            image 'rust:latest'
        }    
    }
    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Build') {
            steps {
                sh 'cargo build --release'
                archiveArtifacts artifacts: 'target/release/*', fingerprint: true
            }
        }
    }
}