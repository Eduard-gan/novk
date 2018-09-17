# novk
Rebel anti-VK project

# Deployment:
 - ### FIXME: Learn to bulid images (or even autobuilds) and publish it on dockerhub. In deploy well only need to pull em, not build because building on production server takes 40 min....
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


## Build problems:
On Arch linux to pip install psycopg2cffi you need to:
 - sudo pacman -S postgresql-libs

## Running outside of Docker
 - pyenv install pypy-3.5-6.0.0
 - pipenv install --python pypy3

## Certbot on server:
 - certbot certonly --webroot -w /var/novk/ssl -d novk.tk
 ### FIXME: Certbot will generate symlinks in live direcrtory. Cherokee needs real files, not symlinks in current setup.
