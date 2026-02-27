function setup()
    default({
        recipe = Recipe.new,
        repository = "https://github.com/mbenoukaiss/ettac.git",
        keep_releases = 3,
        persistent_files = { ".env" },
        persistent_dirs = { "storage" },
    })

    host("test", {
        hostname = "127.0.0.1",
        port = 22,
        user = "bob",
        private_key = env("PRIVATE_KEY"),
        path = "/var/www/ettac",
    })
end

Recipe = {}

function Recipe:new()
    return setmetatable({}, self)
end

function Recipe:describe()
    use(System:new())
end
