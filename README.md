# novk
Rebel anti-VK project

# Deployment:
 - Put your deploying user's public key to root's authorized keys on deployment target computer.
 - Install Docker on it: https://docs.docker.com/install/linux/docker-ce/debian/
 - Install docker-compose(Not from pip, Ansible won't see it :( ):  
    `curl -L "https://github.com/docker/compose/releases/download/1.22.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose`  
    `chmod +x /usr/local/bin/docker-compose`
 - Setup deploying computer(probably your dev PC)  
    `git clone https://github.com/Eduard-gan/novk.git && cd novk`  
    `pipenv install --dev`
     - Deploy project on server  
    `pipenv run ansible-playbook ansible/prod-deploy.yml -i ansible/hosts`
     - Or run it on your local macine  
    `pipenv run ansible-playbook ansible/local-deploy.yml -i ansible/hosts`
     - To clean up:  
    `docker-compose down --rmi local && sudo rm -fr /var/novk`
