pipeline {
    agent { 
        docker {
            image 'rust:latest'
            args '-u root' 
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
                sh 'cargo prisma generate && cargo build --release --bin prisma && cargo build --release --bin crawl-comic-worker'
                archiveArtifacts artifacts: 'target/release/crawl-comic-worker'
                archiveArtifacts artifacts: 'target/release/prisma'
                // compress prisma folder
                sh 'tar -zcvf prisma.tar.gz prisma'
                archiveArtifacts artifacts: 'prisma.tar.gz'
            }
        }
    }
}