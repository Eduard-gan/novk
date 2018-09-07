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

## New Deployment

ansible-playbook ansible/prod-deploy.yml -i ansible/hosts

DOCKER :
 - BEFORE DEPLOY:
        sudo chmod -R 777 /your_project_dir_path/data
        WARNING: NEED TO DEVELOP BACKUP POCEDURE
 - DEPLOY:
        docker-compose up -d --build
 - AFTER DEPLOY:
        docker exec novk_g pipenv run ./manage.py migrate
        docker exec novk_g pipenv run ./manage.py collectstatic --no-input
        docker exec novk_g pipenv run ./manage.py shell -c "from django.contrib.auth.models import User; User.objects.create_superuser('root', 'root@novk.tk', 'lolkekpass')"
