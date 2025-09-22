# Utiliser l'image Nginx officielle
FROM nginx:alpine

# Copier le contenu de dist dans le dossier web par défaut de Nginx
COPY dist /usr/share/nginx/html

# Exposer le port 80
EXPOSE 80

# Nginx démarre automatiquement avec l'image
