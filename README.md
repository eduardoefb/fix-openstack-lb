### Openstack LoadBalancer fix

This script provides a workaround to address OpenStack load balancer pools and members that are stuck in the "PENDING_CREATE" provisioning status. It updates the pool status to "ACTIVE" by directly accessing the OpenStack MySQL database using the octavia user credentials. The password is retrieved from the /etc/octavia/octavia.conf configuration file. Additionally, the script will recreate all affected members.

Requirements:
- Openstack Environment variables should be set: 
    - OS_AUTH_URL
    - OS_USER_DOMAIN_NAME
    - OS_USERNAME
    - OS_PASSWORD
    - OS_PROJECT_NAME
- Valid `/etc/octavia/octavia.conf` configuration file.