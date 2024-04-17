pipeline {
    agent { 
        label 'docker'  
    }
    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }
        stage('Login docker'){
            environment {
                DOCKER_LOGIN_INFO = credentials("ShartubeImageToken")
            }
            steps {
                sh ('echo $DOCKER_LOGIN_INFO_PSW | docker login -u $DOCKER_LOGIN_INFO_USR --password-stdin')
                echo 'Login Completed'
            }
            
        }
        stage('Build Docker Image') {
            steps {
                sh 'docker build -t tritranduc11/crawl-comic-worker .'
            }
        }
        stage('Push Docker Image') {
            steps {
                sh 'docker push tritranduc11/crawl-comic-worker'
            }
        }
    }
    post {
        always { 
            sh "docker logout"
        }
    }
}