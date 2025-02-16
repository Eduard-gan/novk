from __future__ import annotations

from enum import Enum
from os import getenv
from typing import Optional

from dotenv import load_dotenv
from fabric import Connection, task, Config
from invoke import UnexpectedExit

load_dotenv('.env')


class Host(str, Enum):
    LOCAL = 'LOCAL'
    DEV = 'DEV'
    PROD = 'PROD'

    @property
    def hostname(self) -> str:
        if self is self.LOCAL:
            return "localhost"
        if self is self.DEV:
            return getenv("DEV_HOSTNAME")
        else:
            raise NotImplementedError

    @property
    def user(self) -> str:
        if self is self.DEV:
            return getenv("DEV_USER")
        else:
            raise NotImplementedError

    @property
    def password(self) -> Optional[str]:
        if self is self.DEV:
            return getenv("DEV_PASSWORD")
        else:
            raise NotImplementedError

    def get_connection_params(self, user: Optional[str] = None) -> dict:
        params = dict(
            host=self.hostname,
            user=self.user
        )
        if self.password:
            params['connect_kwargs'] = {"password": self.password}
        if user:
            params['user'] = user

        return params

    def run(self, command: str, as_root: bool = False, in_project_dir: bool = False) -> None:
        params = self.get_connection_params(user='root' if as_root else None)

        if in_project_dir:
            command = f"cd {getenv('DEV_PROJECT_DIR')} && {command}"

        with Connection(**params) as connection:
            connection.run(command)


@task
def setup(config: Config, host: str) -> None:
    host = Host(host)

    try:
        host.run('git clone https://')
    except UnexpectedExit as e:
        if 'already exists' in e.result.stderr:
            pass
        else:
            raise


@task
def deploy(config: Config, host: str) -> None:
    host = Host(host)
    host.run('git pull', in_project_dir=True)
    host.run('docker-compose up -d --build', in_project_dir=True)


@task
def check(config: Config, host: str) -> None:
    host = Host(host)
    host.run('command -v git >/dev/null; and echo "Git OK"; or echo "Git is not installed"')
    host.run('systemctl is-active --quiet docker; and echo "Docker OK"; or echo "Docker is not running"')
    host.run('command -v docker-compose >/dev/null; and echo "Docker-compose OK"; or echo "Git is not installed"')
    host.run('command -v poetry >/dev/null; and echo "Poetry OK"; or echo "Poetry is not installed"')
    host.run('if test -d "/srv/novk_reflex"; echo "Project directory OK"; else; echo "Project directory not found"; end')
    host.run('if ping -c 1 duck.com >/dev/null 2>&1; echo "Network OK"; else; echo "Network is down"; end')
    host.run('echo "Environment variables:"; cat /srv/novk_reflex/.env')
