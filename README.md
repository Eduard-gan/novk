# novk
Rebel anti-VK project

На windows требуется установка не тольеко python-magic но и python-magic-bin



# Dployment procedure:

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
