function setup()
    default({
        recipe = Recipe.new,
        repository = "https://github.com/mbenoukaiss/ettac.git",
        keep_releases = 3,
        persistent_files = { ".env" },
        persistent_dirs = { "storage" },
    })

    host("prod", {
        hostname = "127.0.0.1",
        port = 22,
        user = "bob",
        private_key = env("PRIVATE_KEY"),
        path = "/var/www/ettac",
        labels = { "prod" },

        keep_releases = 5,
    })

    host("staging", {
        hostname = "127.0.0.2",
        port = 22,
        user = "alice",
        password = env("PASSWORD"),
        path = "/var/www/ettac",
        labels = { "staging" },
    })
end

Recipe = {}

function Recipe:new()
    return setmetatable({}, self)
end

function Recipe:doctrine_setup()
    remote("runs the command on the server")
end

function Recipe:doctrine_post_migrations()
    remote("runs the command on the server")
end

function Recipe:build_frontend_assets()
    set_timeout(5400)

    self("npm install")
    send("node_modules") -- optionally provide a second argument to tell remote directory
    remote("yarn build")
end

function Recipe:send_email()
    -- whatever
end

function Recipe:describe()
    task(Recipe.doctrine_setup)

    local system = System:new()

    local symfony = Symfony:new({
        version = "7.4",
    })

    local crontab = Crontab:new({
        { "0 0 * * *", "php bin/console app:imports" },
    })

    use(system)
    use(symfony)
    use(crontab)

    after(symfony.doctrine_migrations, symfony.build_frontend_assets)
    after(crontab.setup, self.doctrine_post_migrations)
    remove(symfony.supervisor_restart)

    wrap(symfony.doctrine_migrations, function(inner)
        print("Printing before")
        inner()
        print("Printing after!")
    end)

    catch(self.doctrine_setup, function()
        print("Database is already setup");
        continue(); -- resumes execution after a task failure
    end)

    after(system.fail, self.send_email)
end
