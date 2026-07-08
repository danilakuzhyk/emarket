## Internship details descriptions of emarket.png

### 1. Read docs on internship project and chose FE framework (react, angular, vue, htmx)
Topics for assessment:
* Microservices. Pros and Cons
* REST
* Inter-service communication. Kafka basics. Https basics
* Blockchain basics

### 2. Setup OpenId authorization with rust, keycloak and postgres

Write web api application to setup IAM. Create FE for login, register, forgot password

Required endpoints:
* `POST 'api/users/login'` authorizes user and returns JWT and refresh token
* `POST 'api/users/logout'` logout user
* `POSTS 'api/users/refresh'` refreshes user's JWT token with refresh token
* `POST 'api/users/customers/register'` registers user in keycloak and publishes message to kafka topic 'customer-registered', that customer created an account
* `POST 'api/users/vendors/register'` registers user in keycloak and publishes message to kafka topic 'vendor-registered', that vendor created an account

Topics for assessment:
* Authorization vs Authentication
* What is JWT. JWT Signature
* What is refresh token

### 3. Setup CustomersAPI with postgres database and chosen ORM

Write web api application to maintain customers' profiles. Create FE for customer to logic and displaying for his profile

Required endpoints:
* `GET 'api/customers/profile'` retrieves customer profile (email, avatar_uri, shipment_addresses, billing_address, wallets)
* `PATCH 'api/customer/profile'` updates customer profile
* 'customer-registered' topic subscription, that will create user profile based on payload

Topics for assessment:
* ORM. Pros and Cons
* DB schema migration
* Specific question on chosen ORM
* General question on SQL databases. Indexes, tables, relations etc.
* Kafka topics, partitions etc. producer & consumer

### 4 Setup VendorsAPI with postgres 

Write web api application to maintain vendors' profiles. Create FE for customer to vendor and displaying for his profile

Required endpoints:
* `GET 'api/vendor/profile'` retrieves customer profile (email, avatar_uri, shipment_addresses, wallets)
* `PATCH 'api/vendor/profile'` updates customer profile
* 'vendor-registered' topic subscription, that will create user profile based on payload

Topics for assessment:

### 5 Setup GoodsAPI with chosen NoSQL database (Document/Column oriented)

Write web api for goods. Create FE for customer (can browse available goods to order) & vendor (can manange what he sells)

Required endpoints:
* `POST 'api/goods'` stores goods that vendor can sell
* `GET 'api/goods'` receives list of vendor goods (id can be taken from JWT)
* `GET 'api/goods?name&price&category'` queries goods based on provided filters for customer
* `GET 'api/goods/basket'` receive list of planned orders for customer (id can be taken from JWT)

Topics for assessment:
* NoSQL vs SQL
* CAP theorem
* Specific question on chosen database

### 6 Setup OrdersAPI with postgres db and sqlx (any other plain sql package)

Write web api fore orders. Create FE for vendors (can manage what customer ordered from them) & customer (can order goods)

Required endpoints:
* `POST 'api/orders'` create order, stores customer_id, order_id
* `GET 'api/orders?state'` get customer's orders based on state (plan_to_buy, buyed, shipped etc)
* `POST 'api/orders/cashout'` creates a transactions to "buy" selected goods (does some blockchain stuff). 

Topics for assessment:
* ACID
* SQL Queries: JOINS, GROUP BY, OVER PARTITION BY
* Questions specific to chosen blockchain

### 7 Setup Blob Storage

Setup blob storage service + proxy (or web api app), needed to store files (avatars, goods pictures)

Required endpoints:
* `POST 'api/file'` create file
* `GET 'api/file'` read file

Topics for assessment:
* CDN

### 8 Setup gateway proxy

Setup single entry point for solution (NGINX, Traeffik etc)

Topics for assessment:
* Proxy services

### 9 Setup orchestration

Provide docker-compose file or k8s + helm charts

Topics for assessment:
* Containers
* Orchestration
* Vertical & Horizontal scaling
