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
        port = 7122,
        user = "bob",
        private_key = env("PRIVATE_KEY"),
        path = "/var/www/ettac",
        labels = { "prod" },

        keep_releases = 5,
    })
end

Recipe = {}

function Recipe:new()
    return setmetatable({}, self)
end

function Recipe:describe()
    use(System:new())
end
