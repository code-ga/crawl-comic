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
                sh 'cargo prisma generate && cargo build --release'
                archiveArtifacts artifacts: 'target/release/crawl-comic-worker*', fingerprint: true
            }
        }
    }
}