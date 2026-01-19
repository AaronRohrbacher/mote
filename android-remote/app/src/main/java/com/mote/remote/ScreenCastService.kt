package com.mote.remote

import android.app.*
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.PixelFormat
import android.hardware.display.DisplayManager
import android.hardware.display.VirtualDisplay
import android.media.AudioManager
import android.media.ImageReader
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import android.os.IBinder
import android.util.DisplayMetrics
import android.util.Log
import android.view.WindowManager
import java.io.*
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.atomic.AtomicReference

class ScreenCastService : Service() {
    private var mediaProjection: MediaProjection? = null
    private var virtualDisplay: VirtualDisplay? = null
    private var imageReader: ImageReader? = null
    private var httpServer: Thread? = null
    private var running = true
    
    private val currentFrame = AtomicReference<ByteArray>()
    private val clients = ConcurrentLinkedQueue<Socket>()
    
    private var nsdManager: NsdManager? = null
    private var serviceName: String? = null
    
    private var screenWidth = 800
    private var screenHeight = 480
    private var screenDpi = 160
    private var realScreenWidth = 1080
    private var realScreenHeight = 1920

    override fun onCreate() {
        super.onCreate()
        try {
            createNotificationChannel()
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        try {
            val notification = createNotification()
            startForeground(1, notification)

            intent?.let {
                val resultCode = it.getIntExtra("resultCode", Activity.RESULT_CANCELED)
                @Suppress("DEPRECATION")
                val data: Intent? = if (android.os.Build.VERSION.SDK_INT >= 33) {
                    it.getParcelableExtra("data", Intent::class.java)
                } else {
                    it.getParcelableExtra("data")
                }
                
                if (resultCode == Activity.RESULT_OK && data != null) {
                    setupMediaProjection(resultCode, data)
                    startHttpServer()
                }
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }

        return START_STICKY
    }

    private fun setupMediaProjection(resultCode: Int, data: Intent) {
        try {
            val projectionManager = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
            mediaProjection = projectionManager.getMediaProjection(resultCode, data)

            // Get actual screen dimensions
            val windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager
            val metrics = DisplayMetrics()
            @Suppress("DEPRECATION")
            windowManager.defaultDisplay.getRealMetrics(metrics)
            
            // Store real dimensions for touch scaling
            realScreenWidth = metrics.widthPixels
            realScreenHeight = metrics.heightPixels
            
            // Scale down for performance
            val scale = 0.5f
            screenWidth = (metrics.widthPixels * scale).toInt()
            screenHeight = (metrics.heightPixels * scale).toInt()
            screenDpi = (metrics.densityDpi * scale).toInt()
            
            // Ensure minimum dimensions
            if (screenWidth < 100) screenWidth = 400
            if (screenHeight < 100) screenHeight = 240
            if (screenDpi < 50) screenDpi = 160

            imageReader = ImageReader.newInstance(screenWidth, screenHeight, PixelFormat.RGBA_8888, 2)
            
            virtualDisplay = mediaProjection?.createVirtualDisplay(
                "MoteRemote",
                screenWidth, screenHeight, screenDpi,
                DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,
                imageReader?.surface, null, null
            )

            // Capture frames
            imageReader?.setOnImageAvailableListener({ reader ->
                var image: android.media.Image? = null
                try {
                    image = reader.acquireLatestImage()
                    if (image != null) {
                        val planes = image.planes
                        val buffer = planes[0].buffer
                        val pixelStride = planes[0].pixelStride
                        val rowStride = planes[0].rowStride
                        val rowPadding = rowStride - pixelStride * screenWidth

                        val bitmapWidth = screenWidth + rowPadding / pixelStride
                        if (bitmapWidth > 0 && screenHeight > 0) {
                            val bitmap = Bitmap.createBitmap(
                                bitmapWidth,
                                screenHeight,
                                Bitmap.Config.ARGB_8888
                            )
                            bitmap.copyPixelsFromBuffer(buffer)
                            
                            // Crop to actual size and compress to JPEG
                            val cropped = Bitmap.createBitmap(bitmap, 0, 0, screenWidth, screenHeight)
                            val stream = ByteArrayOutputStream()
                            cropped.compress(Bitmap.CompressFormat.JPEG, 50, stream)
                            currentFrame.set(stream.toByteArray())
                            
                            bitmap.recycle()
                            cropped.recycle()
                        }
                    }
                } catch (e: Exception) {
                    // Ignore frame capture errors
                } finally {
                    image?.close()
                }
            }, null)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    private fun startHttpServer() {
        httpServer = Thread {
            try {
                val serverSocket = ServerSocket(8080)
                
                // Register mDNS service so Pi can find us as "mote.local"
                registerMdnsService()
                
                while (running) {
                    val socket = serverSocket.accept()
                    Thread { handleClient(socket) }.start()
                }
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
        httpServer?.start()
    }
    
    private fun registerMdnsService() {
        try {
            val nsdService = getSystemService(Context.NSD_SERVICE)
            if (nsdService == null) {
                Log.w("MoteRemote", "NSD service not available on this device")
                return
            }
            nsdManager = nsdService as? NsdManager
            if (nsdManager == null) {
                Log.w("MoteRemote", "NSD service not available on this device")
                return
            }
            
            val serviceInfo = NsdServiceInfo().apply {
                serviceName = "mote"
                serviceType = "_http._tcp"
                port = 8080
            }
            
            nsdManager?.registerService(
                serviceInfo,
                NsdManager.PROTOCOL_DNS_SD,
                object : NsdManager.RegistrationListener {
                    override fun onRegistrationFailed(info: NsdServiceInfo?, errorCode: Int) {
                        Log.e("MoteRemote", "mDNS registration failed: $errorCode")
                    }
                    
                    override fun onUnregistrationFailed(info: NsdServiceInfo?, errorCode: Int) {
                        Log.e("MoteRemote", "mDNS unregistration failed: $errorCode")
                    }
                    
                    override fun onServiceRegistered(info: NsdServiceInfo?) {
                        serviceName = info?.serviceName
                        Log.i("MoteRemote", "mDNS registered as: ${info?.serviceName}")
                    }
                    
                    override fun onServiceUnregistered(info: NsdServiceInfo?) {
                        Log.i("MoteRemote", "mDNS unregistered")
                    }
                }
            )
        } catch (e: Exception) {
            Log.e("MoteRemote", "mDNS setup failed", e)
            // Don't crash if mDNS is not available - it's optional
        }
    }

    private fun handleClient(socket: Socket) {
        try {
            val reader = BufferedReader(InputStreamReader(socket.getInputStream()))
            val writer = PrintWriter(socket.getOutputStream(), true)
            val output = socket.getOutputStream()

            val requestLine = reader.readLine() ?: return
            val parts = requestLine.split(" ")
            if (parts.size < 2) return
            
            val path = parts[1]

            when {
                path == "/" || path == "/stream" -> {
                    // MJPEG stream
                    writer.print("HTTP/1.1 200 OK\r\n")
                    writer.print("Content-Type: multipart/x-mixed-replace; boundary=frame\r\n")
                    writer.print("Cache-Control: no-cache\r\n")
                    writer.print("\r\n")
                    writer.flush()

                    clients.add(socket)
                    try {
                        while (running && !socket.isClosed) {
                            val frame = currentFrame.get()
                            if (frame != null) {
                                output.write("--frame\r\n".toByteArray())
                                output.write("Content-Type: image/jpeg\r\n".toByteArray())
                                output.write("Content-Length: ${frame.size}\r\n\r\n".toByteArray())
                                output.write(frame)
                                output.write("\r\n".toByteArray())
                                output.flush()
                            }
                            Thread.sleep(33) // ~30 FPS
                        }
                    } finally {
                        clients.remove(socket)
                    }
                }

                path == "/volume/up" -> {
                    adjustVolume(AudioManager.ADJUST_RAISE)
                    sendResponse(writer, "OK")
                }

                path == "/volume/down" -> {
                    adjustVolume(AudioManager.ADJUST_LOWER)
                    sendResponse(writer, "OK")
                }

                path == "/volume/mute" -> {
                    adjustVolume(AudioManager.ADJUST_TOGGLE_MUTE)
                    sendResponse(writer, "OK")
                }

                path.startsWith("/volume/set/") -> {
                    val level = path.substringAfterLast("/").toIntOrNull()
                    if (level != null) {
                        setVolume(level)
                        sendResponse(writer, "OK")
                    } else {
                        sendResponse(writer, "Invalid volume level", 400)
                    }
                }

                path == "/status" -> {
                    val audioManager = getSystemService(Context.AUDIO_SERVICE) as AudioManager
                    val current = audioManager.getStreamVolume(AudioManager.STREAM_MUSIC)
                    val max = audioManager.getStreamMaxVolume(AudioManager.STREAM_MUSIC)
                    val touchEnabled = TouchInputService.instance != null
                    sendResponse(writer, """{"volume":$current,"max":$max,"width":$screenWidth,"height":$screenHeight,"touchEnabled":$touchEnabled,"realWidth":$realScreenWidth,"realHeight":$realScreenHeight}""")
                }

                // Touch input: /touch?x=100&y=200
                path.startsWith("/touch") -> {
                    val service = TouchInputService.instance
                    if (service == null) {
                        sendResponse(writer, """{"error":"Touch service not enabled. Enable Mote Remote in Accessibility Settings."}""", 503)
                    } else {
                        val params = parseQueryParams(path)
                        val x = params["x"]?.toFloatOrNull()
                        val y = params["y"]?.toFloatOrNull()
                        
                        if (x != null && y != null) {
                            // Scale coordinates from stream resolution to actual screen
                            val scaledX = x * (realScreenWidth.toFloat() / screenWidth)
                            val scaledY = y * (realScreenHeight.toFloat() / screenHeight)
                            service.tap(scaledX, scaledY)
                            sendResponse(writer, """{"ok":true,"x":$scaledX,"y":$scaledY}""")
                        } else {
                            sendResponse(writer, """{"error":"Missing x or y parameter"}""", 400)
                        }
                    }
                }

                // Swipe: /swipe?x1=100&y1=200&x2=300&y2=400
                path.startsWith("/swipe") -> {
                    val service = TouchInputService.instance
                    if (service == null) {
                        sendResponse(writer, """{"error":"Touch service not enabled"}""", 503)
                    } else {
                        val params = parseQueryParams(path)
                        val x1 = params["x1"]?.toFloatOrNull()
                        val y1 = params["y1"]?.toFloatOrNull()
                        val x2 = params["x2"]?.toFloatOrNull()
                        val y2 = params["y2"]?.toFloatOrNull()
                        
                        if (x1 != null && y1 != null && x2 != null && y2 != null) {
                            val scale = realScreenWidth.toFloat() / screenWidth
                            service.swipe(x1 * scale, y1 * scale, x2 * scale, y2 * scale)
                            sendResponse(writer, """{"ok":true}""")
                        } else {
                            sendResponse(writer, """{"error":"Missing coordinates"}""", 400)
                        }
                    }
                }

                // Long press: /longpress?x=100&y=200
                path.startsWith("/longpress") -> {
                    val service = TouchInputService.instance
                    if (service == null) {
                        sendResponse(writer, """{"error":"Touch service not enabled"}""", 503)
                    } else {
                        val params = parseQueryParams(path)
                        val x = params["x"]?.toFloatOrNull()
                        val y = params["y"]?.toFloatOrNull()
                        
                        if (x != null && y != null) {
                            val scale = realScreenWidth.toFloat() / screenWidth
                            service.longPress(x * scale, y * scale)
                            sendResponse(writer, """{"ok":true}""")
                        } else {
                            sendResponse(writer, """{"error":"Missing x or y parameter"}""", 400)
                        }
                    }
                }

                else -> {
                    sendResponse(writer, "Not Found", 404)
                }
            }
        } catch (e: Exception) {
            // Client disconnected
        } finally {
            try { socket.close() } catch (e: Exception) {}
        }
    }

    private fun adjustVolume(direction: Int) {
        val audioManager = getSystemService(Context.AUDIO_SERVICE) as AudioManager
        audioManager.adjustStreamVolume(
            AudioManager.STREAM_MUSIC,
            direction,
            AudioManager.FLAG_SHOW_UI
        )
    }

    private fun setVolume(level: Int) {
        val audioManager = getSystemService(Context.AUDIO_SERVICE) as AudioManager
        val max = audioManager.getStreamMaxVolume(AudioManager.STREAM_MUSIC)
        val clamped = level.coerceIn(0, max)
        audioManager.setStreamVolume(AudioManager.STREAM_MUSIC, clamped, AudioManager.FLAG_SHOW_UI)
    }

    private fun parseQueryParams(path: String): Map<String, String> {
        val params = mutableMapOf<String, String>()
        val queryStart = path.indexOf('?')
        if (queryStart >= 0 && queryStart < path.length - 1) {
            val query = path.substring(queryStart + 1)
            query.split('&').forEach { param ->
                val parts = param.split('=', limit = 2)
                if (parts.size == 2) {
                    params[parts[0]] = parts[1]
                }
            }
        }
        return params
    }

    private fun sendResponse(writer: PrintWriter, body: String, code: Int = 200) {
        val status = when (code) {
            200 -> "OK"
            400 -> "Bad Request"
            404 -> "Not Found"
            else -> "Error"
        }
        writer.print("HTTP/1.1 $code $status\r\n")
        writer.print("Content-Type: application/json\r\n")
        writer.print("Access-Control-Allow-Origin: *\r\n")
        writer.print("Content-Length: ${body.length}\r\n")
        writer.print("\r\n")
        writer.print(body)
        writer.flush()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                "mote_remote",
                "Mote Remote",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(): Notification {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_IMMUTABLE
        )

        return Notification.Builder(this, "mote_remote")
            .setContentTitle("Mote Remote")
            .setContentText("Screen sharing active")
            .setSmallIcon(android.R.drawable.ic_menu_view)
            .setContentIntent(pendingIntent)
            .build()
    }

    override fun onDestroy() {
        running = false
        
        // Unregister mDNS
        try {
            nsdManager?.unregisterService(object : NsdManager.RegistrationListener {
                override fun onRegistrationFailed(info: NsdServiceInfo?, errorCode: Int) {}
                override fun onUnregistrationFailed(info: NsdServiceInfo?, errorCode: Int) {}
                override fun onServiceRegistered(info: NsdServiceInfo?) {}
                override fun onServiceUnregistered(info: NsdServiceInfo?) {}
            })
        } catch (e: Exception) {
            // Ignore cleanup errors
        }
        
        virtualDisplay?.release()
        imageReader?.close()
        mediaProjection?.stop()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}
