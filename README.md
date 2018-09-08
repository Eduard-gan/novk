# novk
Rebel anti-VK project

# Deployment:
 - Put your deploying user's public key to root's authorized keys on deployment target computer.
 - Install Docker on it: https://docs.docker.com/install/linux/docker-ce/debian/
 - And docker-compose as well:  
    `pip install docker-compose`
 - Setup deploying computer(probably your dev PC)  
    `git clone https://github.com/Eduard-gan/novk.git && cd novk`  
    `pipenv install --dev`
     - Edit novk/global_env file for secure custom keys and passwords
     - Deploy project on server  
    `pipenv run ansible-playbook ansible/prod-deploy.yml -i ansible/hosts`
     - Or run it on your local macine  
    `pipenv run ansible-playbook ansible/local-deploy.yml -i ansible/hosts`
    ### FIXME: Run twice because of too slo postgres start. 
     - To clean up:  
    `docker-compose down --rmi local && sudo rm -fr /var/novk`
