# Utiliser l'image Nginx officielle
FROM nginx:alpine

# Copier le contenu de dist dans le dossier web par défaut de Nginx
COPY dist /usr/share/nginx/html

COPY nginx.conf /etc/nginx/conf.d/default.conf
# Exposer le port 80
EXPOSE 80

# Nginx démarre automatiquement avec l'image
