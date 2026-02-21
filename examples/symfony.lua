function setup()
    default({
        recipe = "Recipe",
        repository = "https://github.com/mbenoukaiss/ettac.git",
        keep_releases = 3,
        persistent_files = { ".env" },
        persistent_dirs = { "storage" },
    })

    host("prod", {
        hostname = "127.0.0.1",
        port = 22,
        user = "admin",
        key = env("SSH_PRIVATE_KEY"),
        path = "/var/www/ettac",
        labels = { stage = "prod" },

        -- override or extend defaults as needed
        keep_releases = 5,
    })

    host("staging", {
        hostname = "127.0.0.2",
        port = 22,
        user = "admin",
        password = env("REMOTE_PASSWORD"), -- pulls from env
        path = "/var/www/ettac",
        labels = { stage = "staging" },

        -- override or extend defaults as needed
        keep_releases = 5,
    })
end

Recipe = {}

function Recipe:doctrine_setup()
    remote("runs the command on the server")
end

function Recipe:doctrine_post_migrations()
    remote("runs the command on the server")
end

function Recipe:build_frontend_assets()
    timeout(5400)

    self("npm install")
    setup("node_modules") -- optionally provide a second argument to tell remote directory
    remote("yarn build")
end

function Recipe:send_email()
    -- whatever
end

function Recipe:describe()
    task(Recipe.doctrine_setup)

    use(Symfony)
    use(Crontab)

    after(Symfony.doctrine_migrations, Recipe.build_frontend_assets)
    after(Crontab.doctrine_migrations, Recipe.doctrine_post_migrations)
    remove(Symfony.supervisor_restart)

    wrap(Symfony.doctrine_migrations, function(inner)
        print("Printing before")
        inner()
        print("Printing after!")
    end)

    catch(tasks.doctrine_setup, function()
        print("Database is already setup");
        continue(); -- resumes execution after a task failure
    end)

    after(System.fail, Recipe.send_email)
end

return Recipe
