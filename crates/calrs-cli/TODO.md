# Lister
calrs list                          # tous les items
calrs list --kind event
calrs list --kind task
calrs list --status active
calrs list --kind task --state todo

# Ajouter
calrs add event "Réunion" --start "2026-05-01 14:00" --end "2026-05-01 15:00"
calrs add task "Faire les courses" --deadline "2026-05-03"

# Voir / modifier / supprimer
calrs show 42
calrs delete 42
calrs update 42 --title "Nouveau titre"

# Recherche et vues temporelles
calrs search "réunion"
calrs today
calrs week
calrs date 2026-05-01
calrs range 2026-05-01 2026-05-07

# Profils (sous-commandes)
calrs profile list                  # * devant l'actif
calrs profile create "work"         # ereur si pas de profile creer avant autres cmd
calrs profile delete "work"
calrs profile select "work"

# Flag global pour usage ponctuel
calrs --profile work list