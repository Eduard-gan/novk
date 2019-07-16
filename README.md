# novk
Rebel anti-VK project

# Deployment:
 - ### FIXME: Learn to bulid images (or even autobuilds) and publish it on dockerhub. On deploy we'll only need to pull em, not build because building on production server takes 40 min....
 - Put your deploying user's public key to root's authorized keys on deployment target computer.
 - Install Docker on it: https://docs.docker.com/install/linux/docker-ce/debian/
 - And docker-compose as well:  
    `pip install docker-compose` # WARNING USE EXPLICITLY PIP2 AND WATCH IT INSTALLS docker package in /usr/local/lib/dist-packages/docker
 - Setup deploying computer(probably your dev PC)  
    `git clone https://github.com/Eduard-gan/novk.git && cd novk`  
    `pipenv install --dev`
     - Edit novk/global_env file for secure custom keys and passwords
     - Deploy project on server  
    `pipenv run ansible-playbook ansible/prod-deploy.yml -i ansible/hosts`
     - Or run it on your local macine  
    `pipenv run ansible-playbook ansible/local-deploy.yml -i ansible/hosts`
    ### FIXME: Run twice because of too slow postgres start. 
     - To clean up:  
    `docker-compose down --rmi local && sudo rm -fr /var/novk`


## Build problems:
On Arch linux to pip install psycopg2cffi you need to:
 - sudo pacman -S postgresql-libs

## Running outside of Docker
 - pyenv install pypy-3.5-6.0.0
 - pipenv install --python pypy3

## Certbot on server:

## If Cherokee is not working without cert
 - mkdir -p /etc/letsencrypt/live/novk.ga
 - openssl req -new -x509 -days 365 -nodes -out /etc/letsencrypt/live/novk.ga/fullchain.pem -keyout /etc/letsencrypt/live/novk.ga/privkey.pem
 
## Get real cert with Cherokee
 - pip install certbot
 - certbot certonly --webroot -w /var/novk/ssl -d novk.ga
