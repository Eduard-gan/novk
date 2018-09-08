# novk
Rebel anti-VK project


# OLD Dployment procedure:

1) Backup  
rm -r /backup/*  
cp /var/www/env/novk/db.sqlite3 /backup  
cp -r /var/www/env/novk/static/uploaded /backup  
mv /var/www/env/novk novk_old  

2) Deploy  
cd /var/www/env  
git clone https://github.com/Eduard-gan/novk  
cp /backup/db.sqlite3 novk/  
cp -r /backup/uploaded novk/static  
chown -R www-data:www-data novk  
chmod -R 775 novk  
source /var/www/env/bin/activate  
python novk/manage.py collectstatic  

3) Check and clean  
rm -r /var/www/env/novk_old  
exit  

# New Deployment:
 - First of all add your deploying user's public key to root's authorized keys on deployment target computer.
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
    `docker-compose down --rmi local ; sudo rm -fr /var/novk`
