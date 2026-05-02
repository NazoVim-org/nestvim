return {
  name = "hello-lua",
  version = "0.1.0",
  setup = function(a)
    a.addCommand("hellolua", function()
      a.log("Hello from Lua plugin!")
    end)

    a.on("editor:ready", function()
      a.log("hello-lua plugin ready")
    end)
  end
}